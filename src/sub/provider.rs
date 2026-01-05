// TODO split this down into subfiles
// TODO maybe the traits are pointless, just expose the atomic stuff beneath?

use rand::seq::SliceRandom;

use crate::{ext::{self, err::IgnorableError}, sub::{self, cache::Cache, Loader}};

pub trait Player {
	fn paused(&self) -> bool;
	fn set_paused(&self, val: bool);

	fn resume(&self) {
		self.set_paused(false);
	}

	fn pause(&self) {
		self.set_paused(true);
	}

	fn play_pause(&self) {
		self.set_paused(!self.paused());
	}
}

pub trait Buffer<T> {
	fn progress(&self) -> f32;
	fn seek(&self, pos: f32);

	#[allow(unused)]
	fn is_empty(&self) -> bool {
		self.progress() >= 1.
	}

	fn restart(&self) {
		self.seek(0.);
	}

	fn skip(&self, amount: f32) {
		self.seek(self.progress() + amount.clamp(-1., 1.));
	}
}

pub trait Queue<T> {
	fn current(&self) -> Option<T>;
	fn index(&self) -> usize;
	fn set_index(&self, index: usize);

	fn next(&self);
	fn previous(&self);
	fn reset(&self, data: Vec<T>);

	fn enqueue(&self, x: T);
	fn enqueue_next(&self, x: T);
	#[allow(unused)]
	fn enqueue_at(&self, index: usize, x: T);

	fn get_at(&self, index: usize) -> Option<T>;
	fn view(&self) -> Vec<T>;

	fn dequeue(&self, index: usize);
}

#[derive(Clone)]
pub struct Provider {
	paused: ext::atomic::Flag,
	working: ext::atomic::Flag,
	sink: crate::audio::sink::AudioSink<f32>,
	queue: ext::atomic::Queue<sub::Song>,
	likes: ext::atomic::Sync<Vec<sub::Song>>,
	search: ext::atomic::Sync<Vec<sub::Song>>,
	// ....
	pub artists: ext::atomic::Sync<Vec<sub::Artist>>,
	pub albums: ext::atomic::Sync<Vec<sub::Song>>,
	pub songs: ext::atomic::Sync<Vec<sub::Song>>,
	tx: tokio::sync::mpsc::UnboundedSender<Op>,
}

impl Provider {
	pub fn create(
		cfg: crate::config::Config,
		sink: crate::audio::sink::AudioSink<f32>,
		paused: ext::atomic::Flag,
	) -> (Provider, ProviderWorker) {
		let auth = submarine::auth::AuthBuilder::new(&cfg.auth.username, "v1.16.1")
			.client_name(&cfg.player.device)
			.hashed(&cfg.auth.password);
		let client = submarine::Client::new(&cfg.server.base, auth);

		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		let likes = ext::atomic::Sync::new(Vec::new());
		let queue = ext::atomic::Queue::new(Vec::new());
		let search = ext::atomic::Sync::new(Vec::new());
		let artists = ext::atomic::Sync::new(Vec::new());
		let albums = ext::atomic::Sync::new(Vec::new());
		let songs = ext::atomic::Sync::new(Vec::new());
		let working = ext::atomic::Flag::new(false);
		(
			Provider { 
				tx,
				paused,
				sink: sink.clone(),
				working: working.clone(),
				queue: queue.clone(),
				likes: likes.clone(),
				search: search.clone(),
				artists: artists.clone(),
				albums: albums.clone(),
				songs: songs.clone(),
			},
			ProviderWorker {
				likes,
				working,
				client,
				queue,
				sink,
				search,
				artists,
				albums,
				songs,
				rx,
				cfg,
			},
		)
	}

	pub fn play(&self, song: sub::Id) {
		self.sink.buffer.seek(0);
		if let Some(data) = sub::cache::data().lookup(&song) {
			self.sink.buffer.set(data);
		} else {
			log::warn!("song '{song}' not preloaded, can't play");
			self.tx.send(Op::Load(song)).ignore();
			self.sink.buffer.set(Vec::new());
		}
	}

	pub fn working(&self) -> bool {
		self.working.get()
	}

	pub fn likes(&self) -> Vec<sub::Song> {
		self.likes.get()
	}

	pub fn search_results(&self) -> Vec<sub::Song> {
		self.search.get()
	}

	pub fn shuffle_liked(&self) {
		let mut queue = self.likes.get();
		queue.shuffle(&mut rand::rng());
		self.queue.set(queue);
		if let Some(s) = self.current() {
			self.play(s.id);
		}
	}

	pub fn refresh_likes(&self) {
		self.tx.send(Op::RefreshLikes).ignore();
	}

	pub fn refresh_library(&self) {
		self.tx.send(Op::RefreshArtists).ignore();
		self.tx.send(Op::RefreshAlbums).ignore();
		self.tx.send(Op::RefreshSongs).ignore();
	}

	pub fn search(&self, query: String) {
		self.tx.send(Op::Search(query)).ignore();
	}

	pub fn loading(&self) -> bool {
		self.sink.buffer.len() == 0
	}

	pub fn scrobble(&self, id: sub::Id) {
		self.tx.send(Op::Scrobble(id)).ignore();
	}
}

impl Player for Provider {
	fn set_paused(&self, val: bool) {
		self.paused.set(val);
	}
	fn paused(&self) -> bool {
		self.paused.get()
	}
}

impl Buffer<f32> for Provider {
	fn progress(&self) -> f32 {
		let x = self.sink.buffer.pos() as f32 / self.sink.buffer.len() as f32;
		// TODO wtf is going on here??? .clamp() doesnt work...
		if x.is_nan() {
			return 0.;
		}

		x.clamp(0., 1.)
	}
	fn seek(&self, pos: f32) {
		let off = self.sink.buffer.len() as f32 * pos.clamp(0., 1.);
		self.sink.buffer.seek(off as usize);
	}
}

impl Queue<sub::Song> for Provider {
	fn index(&self) -> usize {
		self.queue.position()
	}
	fn set_index(&self, index: usize) {
		self.queue.set_position(index);
	}

	fn current(&self) -> Option<sub::Song> {
		self.queue.current()
	}

	fn get_at(&self, index: usize) -> Option<sub::Song> {
		self.queue.get(index)
	}

	fn next(&self) {
		if self.progress() > 0.75 && let Some(s) = self.current() {
			self.scrobble(s.id);
		}

		self.queue.advance();
		if let Some(s) = self.queue.current() {
			self.play(s.id);
			notify_rust::Notification::new()
				.summary(&s.title)
				.body(&format!("{} - {}", s.artist.unwrap_or_default(), s.album.unwrap_or_default()))
				// .urgency(notify_rust::Urgency::Low)
				.appname("subtui")
				.show()
				.ignore();
		} else {
			// no upcoming songs, clear buffer and go to silence
			self.sink.buffer.set(Vec::new());
		}
	}
	fn previous(&self) {
		// if it's the start of a song
		if self.progress() < 0.01 {
			// advance in queue, if there's anything next
			if self.index() > 0 {
				self.queue.set_position(self.index() - 1);
				if let Some(s) = self.queue.current() {
					self.play(s.id);
					notify_rust::Notification::new()
						.summary(&s.title)
						.body(&format!("{} - {}", s.artist.unwrap_or_default(), s.album.unwrap_or_default()))
						// .urgency(notify_rust::Urgency::Low)
						.appname("subtui")
						.show()
						.ignore();
				}
			}
		} else {
			// restart this song
			self.restart();

		}
	}

	fn reset(&self, data: Vec<sub::Song>) {
		self.queue.set(data);
	}
	fn enqueue(&self, x: sub::Song) {
		self.queue.insert(self.queue.len(), x);
	}
	fn enqueue_next(&self, x: sub::Song) {
		self.queue.insert(self.queue.position() + 1, x);
	}
	fn enqueue_at(&self, index: usize, x: sub::Song) {
		self.queue.insert(index, x);
	}
	fn dequeue(&self, index: usize) {
		self.queue.remove(index);
	}
	fn view(&self) -> Vec<sub::Song> {
		self.queue.view()
	}
}


enum Op {
	RefreshLikes,
	RefreshArtists,
	RefreshAlbums,
	RefreshSongs,
	Load(sub::Id),
	Search(String),
	Scrobble(sub::Id),
}

pub struct ProviderWorker {
	rx: tokio::sync::mpsc::UnboundedReceiver<Op>,
	sink: crate::audio::sink::AudioSink<f32>,
	likes: ext::atomic::Sync<Vec<sub::Song>>,
	client: submarine::Client,
	working: ext::atomic::Flag,
	queue: ext::atomic::Queue<sub::Song>,
	search: ext::atomic::Sync<Vec<sub::Song>>,
	cfg: crate::config::Config,
	// TODO overdoing this a bit... need a better way than 2 channels, ouchh
	artists: ext::atomic::Sync<Vec<sub::Artist>>,
	albums: ext::atomic::Sync<Vec<sub::Song>>,
	songs: ext::atomic::Sync<Vec<sub::Song>>,
}

impl ProviderWorker {
	pub async fn work(mut self) {
		let mut last_fetch = std::time::SystemTime::now();

		// TODO ughh yet another bunch of copies.......
		let _client = self.client.clone();
		let _sink = self.sink.clone();
		let _queue = self.queue.clone();
		let _working = self.working.clone();
		let _preload = self.cfg.player.preload;
		let (tx, mut rx) = tokio::sync::mpsc::channel(10);
		tokio::spawn(async move {
			loop {
				tokio::select! {
					biased;

					res = rx.recv() => match res {
						None => break,
						Some(id) => {
							_working.set(true);
							Self::preload(id, &_client, &_sink, &_queue).await;
							_working.set(false);
						},
					},

					_ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {},

				}

				_working.set(true);
				for i in 0.._preload {
					if let Some(song) = _queue.get(_queue.position() + i) {
						Self::preload(song.id, &_client, &_sink, &_queue).await;
					}
				}
				_working.set(false);
			}
		});

		while let Some(op) = self.rx.recv().await {
			match op {
				Op::Load(id) => tx.send(id).await.ignore(),
				Op::RefreshLikes => {
					self.reload_likes().await;
					last_fetch = std::time::SystemTime::now();
				},
				Op::Search(query) => {
					log::info!("searching '{query}'");
					match self.client.search3(
						query,
						Some(50),
						None,
						Some(50),
						None,
						Some(50),
						None,
						None::<String>,
					)
						.await
					{
						Err(e) => log::error!("error searching: {e}"),
						Ok(x) => self.search.set(x.song),
					}
				},
				Op::RefreshArtists => {
					log::info!("refreshing artists...");
					match self.client.all_artists().await {
						Err(e) => log::error!("error loading all artists: {e}"),
						Ok(artists) => self.artists.set(artists),
					}
				},
				Op::RefreshSongs => {
					log::info!("refreshing songs...");
					match self.client.all_songs().await {
						Err(e) => log::error!("error loading all songs: {e}"),
						Ok(songs) => self.songs.set(songs),
					}
				},
				Op::RefreshAlbums => {
					log::info!("refreshing albums...");
					match self.client.all_albums().await {
						Err(e) => log::error!("error loading all albums: {e}"),
						Ok(albums) => self.albums.set(albums),
					}
				},
				Op::Scrobble(id) => {
					log::info!("scrobbling play of '{id}'");
					if let Err(e) = self.client.scrobble(vec![(id, None)], Some(true)).await {
						log::error!("error scrobbling song: {e}");
					}
				},
			}

			if std::time::SystemTime::now() > last_fetch + std::time::Duration::from_secs(300) {
				self.reload_likes().await;
				last_fetch = std::time::SystemTime::now();
			}

		}
		log::info!("provider worker quitting");
	}

	async fn preload(id: sub::Id, client: &submarine::Client, sink: &crate::audio::sink::AudioSink<f32>, queue: &ext::atomic::Queue<sub::Song>) {
		match sub::cache::data().prime(&id, client.clone()).await {
			Err(e) => log::error!("error preloading data for song '{id}': {e}"),
			Ok(()) => {
				if let Some(s) = queue.current() && s.id == id && sink.buffer.len() == 0 {
					let data = sub::cache::data().lookup(&id).expect("just primed");
					log::info!("setting playback buffer");
					sink.buffer.set(data);
				}
			},
		}
		if let Err(e) = sub::cache::meta().prime(&id, client.clone()).await {
			log::error!("error preloading meta for song '{id}': {e}")
		}
	}

	async fn reload_likes(&self) {
		log::info!("reloading likes");
		self.working.set(true);
		match self.client.get_starred(None::<String>).await {
			Ok(data) => self.likes.set(data.song),
			Err(e) => log::error!("error fetching likes: {e}"),
		}
		self.working.set(false);
	}
}

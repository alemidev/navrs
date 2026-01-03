use std::sync::{Arc, OnceLock};

use dashmap::DashMap;
use rand::seq::SliceRandom;
use tokio::sync::{mpsc, watch};

use crate::ext::{atomic, err::IgnorableError};

pub type Song = submarine::data::Child;
pub type Id = String;

pub type SubResult<T> = Result<T, submarine::SubsonicError>;

// TODO huge trait, more default or split it down!!
pub trait MusicProvider<T>: Send + Clone {
	fn data(&self, len: usize) -> Option<Vec<T>>; // TODO cloneless??

	fn current_song(&self) -> Option<Song>;
	fn likes(&self) -> Vec<Song>;

	fn progress(&self) -> f32;
	fn working(&self) -> bool;
	fn running(&self) -> bool;

	fn set_pause(&self, state: bool);
	fn play(&self, song: Id);
	fn next(&self);

	fn queue(&self) -> Vec<Song>;
	fn enqueue(&self, songs: Vec<Song>);
	fn play_next(&self, song: Song);

	#[deprecated = "can't be really done without locking, need higher-level methods maybe"]
	fn pop_queue(&self, idx: usize) -> Option<Song>;
	fn clear_queue(&self);
	fn seek(&self, percentage: f32);

	fn toggle(&self) {
		self.set_pause(!self.running());
	}

	fn pause(&self) {
		self.set_pause(true);
	}

	fn resume(&self) {
		self.set_pause(false);
	}

	fn restart(&self) {
		self.seek(0.);
	}

	fn skip(&self, amount: f32) {
		self.seek(
			self.progress() + amount.clamp(-1., 1.)
		);
	}

	fn shuffle_liked(&self) {
		let mut queue = self.likes();
		queue.shuffle(&mut rand::rng());
		self.clear_queue();
		self.enqueue(queue);
		self.next();
	}
}


#[derive(Clone)]
pub struct SubsonicProvider(Arc<SubsonicProviderHandle>);
struct SubsonicProviderHandle {
	song: watch::Receiver<Option<Song>>,
	likes: watch::Receiver<Vec<Song>>,
	queue: atomic::Queue<Song>,
	buffer: atomic::Buffer<f32>,
	op: mpsc::UnboundedSender<Op>,
	running: atomic::Flag,
	working: atomic::Flag,
}

struct SubsonicProviderActor {
	client: submarine::Client,
	song: watch::Sender<Option<Song>>,
	likes: watch::Sender<Vec<Song>>,
	queue: atomic::Queue<Song>,
	buffer: atomic::Buffer<f32>,
	op: mpsc::UnboundedReceiver<Op>,
	running: atomic::Flag,
	working: atomic::Flag,
}

impl SubsonicProvider {
	pub fn create(host: String, username: String, password: String) -> (Self, impl std::future::Future<Output = impl Send> + Send) {
		let auth = submarine::auth::AuthBuilder::new(username, "v1.16.1")
			.client_name("subtui")
			.hashed(&password);
		let client = submarine::Client::new(&host, auth);
		let buffer = atomic::Buffer::new(vec![0f32; 4096]); // TODO sizing?
		let queue = atomic::Queue::new(vec![]);
		let (song_tx, song_rx) = watch::channel(None);
		let (likes_tx, likes_rx) = watch::channel(Vec::new());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let running = atomic::Flag::new(false);
		let working = atomic::Flag::new(false);

		op_tx.send(Op::RefreshLikes).ignore();

		let worker = SubsonicProviderActor {
			client,
			song: song_tx,
			likes: likes_tx,
			queue: queue.clone(),
			buffer: buffer.clone(),
			op: op_rx,
			running: running.clone(),
			working: working.clone(),
		};

		(
			Self(Arc::new(SubsonicProviderHandle {
				song: song_rx,
				likes: likes_rx,
				buffer,
				op: op_tx,
				queue,
				running,
				working
			})),
			async move {
				worker.work().await
			}
		)
	}

	// TODO MEHHHHHH
	pub fn refresh_likes(&self) {
		self.0.op.send(Op::RefreshLikes).ignore();
	}
}

impl MusicProvider<f32> for SubsonicProvider {
	fn data(&self, len: usize) -> Option<Vec<f32>> {
		if !self.0.running.get() {
			return None;
		}

		if len > self.0.buffer.len() {
			return None;
		}

		// finished, load next?
		if self.0.buffer.offset() + len > self.0.buffer.len() {
			if self.0.running.get() {
				self.next();
				self.0.running.toggle();
			}
			return None;
		}

		self.0.buffer.try_read(len)
	}

	fn current_song(&self) -> Option<Song> {
		self.0.song.borrow().clone()
	}

	fn likes(&self) -> Vec<Song> {
		self.0.likes.borrow().clone()
	}

	fn progress(&self) -> f32 {
		if self.0.song.borrow().is_none() {
			return 0.;
		}
		self.0.buffer.progress()
	}

	fn play(&self, song: Id) {
		self.0.op.send(Op::PlaySong(song)).ignore();
		if let Some(song) = self.0.queue.peek() {
			self.0.op.send(Op::Preload(song.id)).ignore();
		}
	}

	fn working(&self) -> bool {
		self.0.working.get()
	}

	fn running(&self) -> bool {
		self.0.running.get()
	}

	fn set_pause(&self, state: bool) {
		self.0.running.set(state);
	}

	fn enqueue(&self, songs: Vec<Song>) {
		self.0.op.send(Op::Enqueue(songs, false)).ignore();
	}

	fn play_next(&self, song: Song) {
		self.0.op.send(Op::Enqueue(vec![song], true)).ignore();
	}

	fn clear_queue(&self) {
		self.0.op.send(Op::ClearQueue).ignore();
	}

	fn queue(&self) -> Vec<Song> {
		self.0.queue.snapshot()
	}

	fn pop_queue(&self, idx: usize) -> Option<Song> {
		let res = self.0.queue.snapshot().get(idx).cloned();
		self.0.op.send(Op::Dequeue(idx)).ignore();
		res
	}

	fn next(&self) {
		self.0.op.send(Op::Next).ignore();
	}

	fn seek(&self, percentage: f32) {
		self.0.op.send(Op::Seek(percentage)).ignore();
	}
}

enum Op {
	PlaySong(Id),
	Preload(Id),
	Enqueue(Vec<Song>, bool),
	Dequeue(usize),
	Seek(f32),
	Next,
	ClearQueue,
	RefreshLikes,
}

fn data_cache() -> &'static DashMap<String, Vec<f32>> {
	static CACHE: OnceLock<DashMap<String, Vec<f32>>> = OnceLock::new();
	CACHE.get_or_init(DashMap::default)
}

fn metadata_cache() -> &'static DashMap<String, Song> {
	static CACHE: OnceLock<DashMap<String, Song>> = OnceLock::new();
	CACHE.get_or_init(DashMap::default)
}

async fn preload(id: String, client: &submarine::Client) -> SubResult<Song> {
	let song = match metadata_cache().get(&id) {
		Some(s) => s.value().clone(),
		None => {
			client.get_song(&id).await?
		},
	};

	match data_cache().get(&id) {
		Some(_) => Ok(song),
		None => {
			log::info!("streaming {id}");
			let data = client.stream(
				&id,
				Some(320),
				Some("subtui"),
				None,
				None::<String>,
				None,
				None
			)
				.await?;
			let mut decoder = rmp3::Decoder::new(&data);
			let mut out = Vec::new();
			while let Some(frame) = decoder.next() {
				match frame {
					rmp3::Frame::Audio(audio) => {
						// TODO is there a push_all??
						for sample in audio.samples() {
							out.push(*sample);
						}
					}
					rmp3::Frame::Other(_data) => {},
				}
			}

			data_cache().insert(id, out.clone());
			Ok(song)
		},
	}
}

impl SubsonicProviderActor {
	async fn work(mut self) {
		while let Some(op) = self.op.recv().await {
			self.working.set(true);
			match op {
				Op::Preload(id) => {
					if let Err(e) = preload(id.clone(), &self.client).await {
						log::error!("error preloading song {id}: {e}");
					}
				},
				Op::PlaySong(id) => {
					match preload(id, &self.client).await {
						Err(e) => log::error!("could not preload song for playing: {e}"),
						Ok(song) => if let Some(data) = data_cache().get(&song.id) {
							self.play(song, data.value().clone()).await;
						},
					}
				}
				Op::RefreshLikes => match self.client.get_starred(None::<String>).await {
					Ok(res) => { self.likes.send(res.song).ignore(); },
					Err(e) => log::error!("error fetching likes: {e}"),
				},
				Op::Enqueue(childs, front) => if front {
					for s in childs {
						self.queue.push_front(s).await;
					}
				} else {
					for s in childs {
						self.queue.push(s).await;
					}
				},
				Op::Dequeue(idx) => { self.queue.remove(idx).await; },
				Op::Seek(pos) => self.buffer.seek(pos).await,
				Op::Next => if let Some(song) = self.queue.pop().await {
					match preload(song.id, &self.client).await {
						Err(e) => log::error!("could not preload next song for playing: {e}"),
						Ok(song) => {
							if let Some(data) = data_cache().get(&song.id) {
								self.song.send(Some(song.clone())).ignore();
								self.play(song, data.value().clone()).await;
							}
						},
					}
				},
				Op::ClearQueue => self.queue.clear().await,
			}
			self.working.set(false);
		}
		log::warn!("closing worker loop");
	}

	async fn play(&self, song: Song, data: Vec<f32>) {
		self.song.send(Some(song.clone())).ignore();
		if let Err(e) = libnotify::Notification::new(
			&song.title,
			Some(format!("{} - {}", song.artist.as_deref().unwrap_or_default(), song.album.as_deref().unwrap_or_default()).as_str()),
			Some("music"),
		)
			.show()
		{
			log::error!("error showing notification: {e}");
		}

		self.running.set(true);
	
		self.buffer.set(data).await;
		log::info!("playing {song:?}");
	}
}


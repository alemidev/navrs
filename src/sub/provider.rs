use rand::seq::SliceRandom;

use crate::{audio::api::Buffer, ext::{self, err::IgnorableError}, sub::{self, cache::Cache}};

#[derive(Clone)]
pub struct Provider {
	pub working: ext::atomic::Flag,
	pub player: crate::audio::sink::AudioPlayer,
	pub queue: ext::atomic::Queue<sub::Song>,
	pub likes: ext::atomic::Sync<Vec<sub::Song>>,
	pub search: ext::atomic::Sync<Vec<sub::Song>>,
	// ....
	pub artists: ext::atomic::Sync<Vec<sub::Artist>>,
	pub albums: ext::atomic::Sync<Vec<sub::Song>>,
	pub songs: ext::atomic::Sync<Vec<sub::Song>>,
	tx: tokio::sync::mpsc::UnboundedSender<sub::worker::Op>,
}

impl Provider {
	pub fn create(
		cfg: crate::config::Config,
		player: crate::audio::sink::AudioPlayer,
	) -> (Provider, sub::worker::ProviderWorker) {
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
				player: player.clone(),
				working: working.clone(),
				queue: queue.clone(),
				likes: likes.clone(),
				search: search.clone(),
				artists: artists.clone(),
				albums: albums.clone(),
				songs: songs.clone(),
			},
			sub::worker::ProviderWorker {
				likes,
				working,
				client,
				queue,
				player,
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
		self.tx.send(sub::worker::Op::UpdateMPRIS).ignore();
		if let Some((data, sample_rate)) = sub::cache::data().lookup(&song) {
			if let Err(e) = self.player.play(data, sample_rate) {
				log::error!("error playing song: {e}");
			}
		} else {
			self.player.clear();
			log::warn!("song '{song}' not preloaded, can't play");
		}
	}

	pub fn shuffle_liked(&self) {
		let mut queue = self.likes.get();
		queue.shuffle(&mut rand::rng());
		self.queue.set(queue);
		if let Some(s) = self.queue.current() {
			self.play(s.id);
		}
	}

	pub fn go_next(&self) {
		if self.player.progress() > 0.75 && let Some(s) = self.queue.current() {
			self.scrobble(s.id);
		}

		self.queue.advance();
		if let Some(s) = self.queue.current() {
			self.play(s.id);
			// TODO have one single updated notification
			notify_rust::Notification::new()
				.summary(&s.title)
				.body(&format!("{} - {}", s.artist.unwrap_or_default(), s.album.unwrap_or_default()))
				// .urgency(notify_rust::Urgency::Low)
				.appname("subtui")
				.show()
				.ignore();
		} else {
			// no upcoming songs, clear buffer and go to silence
			self.player.clear();
		}
	}

	pub fn go_previous(&self) {
		// if it's the start of a song
		if self.player.progress() < 0.01 {
			// advance in queue, if there's anything next
			if self.queue.index() > 0 {
				self.queue.set_index(self.queue.index() - 1);
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
			self.player.restart();
		}
	}

	pub fn reset(&self, data: Vec<sub::Song>) {
		self.queue.set(data.clone());
		if let Some(f) = data.first() {
			self.play(f.id.clone());
		} else {
			// cleared the queue, stop playback
			self.player.clear();
		}
	}


	// TODO these are not really scalable... should we just expose the Op struct?

	pub fn refresh_likes(&self) {
		self.tx.send(sub::worker::Op::RefreshLikes).ignore();
	}

	pub fn refresh_library(&self) {
		self.tx.send(sub::worker::Op::RefreshArtists).ignore();
		self.tx.send(sub::worker::Op::RefreshAlbums).ignore();
		self.tx.send(sub::worker::Op::RefreshSongs).ignore();
	}

	pub fn search(&self, query: String) {
		self.tx.send(sub::worker::Op::Search(query)).ignore();
	}

	pub fn scrobble(&self, id: sub::Id) {
		self.tx.send(sub::worker::Op::Scrobble(id)).ignore();
	}
}


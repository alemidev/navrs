use rand::seq::SliceRandom;

use crate::{ext::{self, err::IgnorableError}, sub::{self, cache::Cache}};

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
	fn enqueue_at(&self, index: usize, x: T);

	fn get_at(&self, index: usize) -> Option<T>;
	fn view(&self) -> Vec<T>;

	fn dequeue(&self, index: usize);
}

pub trait Cached<T> {
	
}

#[derive(Clone)]
pub struct Provider {
	paused: ext::atomic::Flag,
	working: ext::atomic::Flag,
	sink: crate::audio::sink::AudioSink<f32>,
	queue: ext::atomic::Queue<sub::Song>,
	likes: ext::atomic::Sync<Vec<sub::Song>>,
	tx: tokio::sync::mpsc::UnboundedSender<Op>,
}

impl Provider {
	pub fn create(
		client: submarine::Client,
		sink: crate::audio::sink::AudioSink<f32>,
		paused: ext::atomic::Flag,
	) -> (Provider, ProviderWorker) {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		let likes = ext::atomic::Sync::new(Vec::new());
		let queue = ext::atomic::Queue::new(Vec::new());
		let working = ext::atomic::Flag::new(false);
		(
			Provider { 
				tx,
				paused,
				sink: sink.clone(),
				working: working.clone(),
				queue: queue.clone(),
				likes: likes.clone(),
			},
			ProviderWorker {
				likes,
				working,
				client,
				queue,
				sink,
				rx,
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

		// TODO checking here is kind of awful!!!
		if x >= 1. {
			self.next();
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
		self.queue.advance();
		if let Some(s) = self.queue.current() {
				self.play(s.id);
		} else {
			// no upcoming songs, clear buffer and go to silence
			self.sink.buffer.set(Vec::new());
		}
	}
	fn previous(&self) {
		if self.index() > 0 {
			self.queue.set_position(self.index() - 1);
			if let Some(s) = self.queue.current() {
				self.play(s.id);
			}
		}
	}

	fn reset(&self, data: Vec<sub::Song>) {
		self.queue.set(data);
	}
	fn enqueue(&self, x: sub::Song) {
		self.queue.insert(self.queue.len(), x);
	}
	fn enqueue_next(&self, x: sub::Song) {
		self.queue.insert(self.queue.position(), x);
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
	Load(sub::Id),
}

pub struct ProviderWorker {
	rx: tokio::sync::mpsc::UnboundedReceiver<Op>,
	sink: crate::audio::sink::AudioSink<f32>,
	likes: ext::atomic::Sync<Vec<sub::Song>>,
	client: submarine::Client,
	working: ext::atomic::Flag,
	queue: ext::atomic::Queue<sub::Song>,
}

impl ProviderWorker {
	pub async fn work(mut self) {
		let mut last_fetch = std::time::SystemTime::now();
		loop {
			tokio::select! {
				biased;

				op = self.rx.recv() => {
					match op {
						None => break,
						Some(Op::Load(id)) => self.preload(id).await,
						Some(Op::RefreshLikes) => {
							self.reload_likes().await;
							last_fetch = std::time::SystemTime::now();
						}
					}
				},

				_ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {},

			}

			if let Some(current) = self.queue.current() {
				self.preload(current.id).await;
			}
			if let Some(next) = self.queue.next() {
				self.preload(next.id).await;
			}

			for i in 0..5 {
				if let Some(song) = self.queue.get(self.queue.position() + i) {
					self.preload(song.id).await;
				}
			}

			if std::time::SystemTime::now() > last_fetch + std::time::Duration::from_secs(300) {
				self.reload_likes().await;
				last_fetch = std::time::SystemTime::now();
			}

		}
		log::info!("provider worker quitting");
	}

	async fn preload(&self, id: sub::Id) {
		self.working.set(true);
		match sub::cache::data().load(&id, self.client.clone()).await {
			Err(e) => log::error!("error preloading data for song '{id}': {e}"),
			Ok(data) => {
				if let Some(s) = self.queue.current() && s.id == id && self.sink.buffer.len() == 0 {
					self.sink.buffer.set(data);
				}
			},
		}
		if let Err(e) = sub::cache::meta().load(&id, self.client.clone()).await {
			log::error!("error preloading meta for song '{id}': {e}")
		}
		self.working.set(false);
	}

	async fn reload_likes(&self) {
		self.working.set(true);
		match self.client.get_starred(None::<String>).await {
			Ok(data) => self.likes.set(data.song),
			Err(e) => log::error!("error fetching likes: {e}"),
		}
		self.working.set(false);
	}
}

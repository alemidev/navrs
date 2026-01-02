use std::{collections::VecDeque, sync::{Arc, OnceLock}};

use dashmap::DashMap;
use rand::seq::SliceRandom;
use sunk::{Streamable, song::Song};

// TODO huge trait, more default or split it down!!
pub trait MusicProvider<T>: Send + Clone {
	fn data(&self, len: usize) -> Option<Vec<T>>; // TODO cloneless??

	fn current_song(&self) -> Option<sunk::song::Song>;
	fn likes(&self) -> Vec<sunk::song::Song>;

	fn progress(&self) -> f32;
	fn working(&self) -> bool;
	fn running(&self) -> bool;

	fn set_pause(&self, state: bool);
	fn play(&self, song: sunk::song::Song);
	fn next(&self) -> bool;

	fn queue(&self) -> Vec<sunk::song::Song>;
	fn enqueue(&self, songs: Vec<sunk::song::Song>);
	fn play_next(&self, song: sunk::song::Song);
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
pub struct SubsonicProvider(Arc<SubsonicProviderInner>);
struct SubsonicProviderInner {
	client: sunk::Client,
	buffer: std::sync::Mutex<SyncBuffer>,
	song: std::sync::RwLock<Option<Song>>,
	likes: std::sync::RwLock<SyncLikes>,
	tx: std::sync::mpsc::Sender<Op>,
	working: Arc<std::sync::atomic::AtomicBool>,
	running: Arc<std::sync::atomic::AtomicBool>,
	queue: std::sync::RwLock<VecDeque<sunk::song::Song>>,
}

struct SyncLikes {
	songs: Vec<Song>,
	updated: std::time::SystemTime,
}

struct SyncBuffer {
	buf: Vec<f32>,
	offset: usize,
}

impl SubsonicProvider {
	pub fn connect(host: &str, username: &str, password: &str) -> Result<Self, sunk::Error> {
		let client =
			sunk::Client::new(host, username, password)?.with_target(sunk::Version::from("1.14.0"));
		println!(
			"connected: ver {} - target {}",
			client.ver, client.target_ver
		);
		let buf = vec![0f32; 4096]; // TODO must be big enough
		let offset = 0;
		let buffer = std::sync::Mutex::new(SyncBuffer { buf, offset });
		let song = std::sync::RwLock::new(None);
		let queue = std::sync::RwLock::new(VecDeque::new());
		let likes = std::sync::RwLock::new(SyncLikes {
			songs: vec![],
			updated: std::time::SystemTime::UNIX_EPOCH,
		});
		let (tx, rx) = std::sync::mpsc::channel();
		let working = Arc::new(std::sync::atomic::AtomicBool::new(false));
		let running = Arc::new(std::sync::atomic::AtomicBool::new(false));

		let inner = Arc::new(SubsonicProviderInner {
			client,
			buffer,
			song,
			likes,
			tx,
			running,
			working: working.clone(),
			queue,
		});
		let ctx = inner.clone();

		std::thread::spawn(move || work(ctx, rx, working));

		Ok(Self(inner))
	}

	pub fn random_song(&mut self) -> Result<Option<Song>, sunk::Error> {
		let songs = Song::random(&self.0.client, 1)?.into_iter().next();
		if let Some(song) = songs {
			self.play(song.clone());
			Ok(Some(song))
		} else {
			Ok(None)
		}
	}
}

impl MusicProvider<f32> for SubsonicProvider {
	fn data(&self, len: usize) -> Option<Vec<f32>> {
		if !self.0.running.load(std::sync::atomic::Ordering::Relaxed) {
			return None;
		}

		let mut buffer = self.0.buffer.lock().unwrap();

		if len > buffer.buf.len() {
			return None;
		}

		// finished, load next?
		if buffer.offset + len > buffer.buf.len() {
			if self.0.running.load(std::sync::atomic::Ordering::Relaxed) {
				self.next();
				self.0.running.store(false, std::sync::atomic::Ordering::Relaxed);
			}
			return None;
		}

		buffer.offset += len;
		let data = buffer.buf[buffer.offset - len..buffer.offset].to_vec();

		Some(data)
	}

	fn current_song(&self) -> Option<sunk::song::Song> {
		self.0.song.read().unwrap().clone()
	}

	fn likes(&self) -> Vec<sunk::song::Song> {
		if std::time::SystemTime::now()
			> self.0.likes.read().unwrap().updated + std::time::Duration::from_secs(300)
		{
			self.0.tx.send(Op::RefreshLikes);
		}

		self.0.likes.read().unwrap().songs.clone()
	}

	fn progress(&self) -> f32 {
		if self.0.song.read().unwrap().is_none() {
			return 0.;
		}
		let buf = self.0.buffer.lock().unwrap();
		(buf.offset as f32 / buf.buf.len() as f32).clamp(0., 1.)
	}

	fn play(&self, song: sunk::song::Song) {
		self.0.tx.send(Op::PlaySong(song));
		if let Some(song) = self.0.queue.read().unwrap().get(0) {
			self.0.tx.send(Op::Preload(song.clone()));
		}
	}

	fn working(&self) -> bool {
		self.0.working.load(std::sync::atomic::Ordering::Relaxed)
	}

	fn running(&self) -> bool {
		self.0.running.load(std::sync::atomic::Ordering::Relaxed)
	}

	fn set_pause(&self, state: bool) {
		self.0
			.running
			.store(state, std::sync::atomic::Ordering::Relaxed);
	}

	fn enqueue(&self, songs: Vec<sunk::song::Song>) {
		let mut q = self.0.queue.write().unwrap();
		for s in songs {
			q.push_back(s);
		}
	}

	fn play_next(&self, song: sunk::song::Song) {
		self.0.queue.write().unwrap().push_front(song);
	}

	fn clear_queue(&self) {
		self.0.queue.write().unwrap().clear();
	}

	fn queue(&self) -> Vec<sunk::song::Song> {
		// TODO lmao whats this can it be better?
		self.0.queue.read().unwrap().iter().cloned().collect()
	}

	fn next(&self) -> bool {
		let res = self.0.queue.write().unwrap().pop_front();
		if let Some(song) = res {
			self.play(song);
			return true;
		}
		false
	}

	fn seek(&self, percentage: f32) {
		let mut buffer = self.0.buffer.lock().unwrap();
		let total = buffer.buf.len() as f32;
		buffer.offset = (total * percentage) as usize;
	}
}

enum Op {
	PlaySong(sunk::song::Song),
	Preload(sunk::song::Song),
	RefreshLikes,
}

fn cache() -> &'static DashMap<sunk::id::Id, Vec<f32>> {
	static CACHE: OnceLock<DashMap<sunk::id::Id, Vec<f32>>> = OnceLock::new();
	CACHE.get_or_init(DashMap::default)
}

fn preload(song: &mut sunk::song::Song, client: &sunk::Client) -> bool {
	if !cache().contains_key(&song.id) {
		log::info!("preloading {song}");
		let mut temp = Vec::new();
		song.set_max_bit_rate(320); // TODO make configurable
		song.set_transcoding("mp3");
		match song.stream(client) {
			Err(e) => { log::error!("error requesting song stream: {e}"); false },
			Ok(mut reader) => match reader.read_to_end(&mut temp) {
				Err(e) => { log::error!("error copying streamed data: {e}"); false },
				Ok(_n) => {
					// TODO this is small and convenient but buggy: switch to symphonia?
					let mut decoder = rmp3::Decoder::new(&temp);
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

					cache().insert(song.id.clone(), out);
					true
				},
			},
		}
	} else {
		true
	}
}

fn work(
	ctx: Arc<SubsonicProviderInner>,
	rx: std::sync::mpsc::Receiver<Op>,
	working: Arc<std::sync::atomic::AtomicBool>,
) {
	while let Ok(op) = rx.recv() {
		working.store(true, std::sync::atomic::Ordering::Relaxed);
		match op {
			Op::Preload(mut song) => {
				preload(&mut song, &ctx.client);
			},
			Op::PlaySong(mut song) => {
				preload(&mut song, &ctx.client);

				if let Some(data) = cache().get(&song.id) {
					*ctx.song.write().unwrap() = Some(song.clone());

					if let Err(e) = libnotify::Notification::new(
						&song.title,
						Some(format!("{} - {}", song.artist.as_deref().unwrap_or_default(), song.album.as_deref().unwrap_or_default()).as_str()),
						Some("music"),
					)
						.show()
					{
						log::error!("error showing notification: {e}");
					}

					ctx.running
						.store(true, std::sync::atomic::Ordering::Relaxed);

					log::info!("playing {song}");
					let mut buffer = ctx.buffer.lock().unwrap();
					buffer.offset = 0;
					buffer.buf = data.value().clone();
				}
			}
			Op::RefreshLikes => match ctx.client.starred(1) {
				Ok(res) => {
					let mut guard = ctx.likes.write().unwrap();
					guard.songs = res.songs;
					guard.updated = std::time::SystemTime::now();
				}
				Err(e) => log::error!("error fetching likes: {e}"),
			},
		}
		working.store(false, std::sync::atomic::Ordering::Relaxed);
	}
	log::warn!("closing worker loop");
}

use std::sync::Arc;

use sunk::{Streamable, song::Song};

pub trait MusicProvider<T>: Send + Clone {
	fn data(&self, len: usize) -> Option<Vec<T>>; // TODO cloneless??
	fn song(&self) -> Option<sunk::song::Song>;
	fn likes(&self) -> Vec<sunk::song::Song>;
	fn progress(&self) -> f32;
	fn working(&self) -> bool;
	fn play(&self, song: sunk::song::Song);
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
		let client = sunk::Client::new(host, username, password)?.with_target(sunk::Version::from("1.14.0"));
		println!("connected: ver {} - target {}", client.ver, client.target_ver);
		let buf = vec![0f32; 4096]; // TODO must be big enough
		let offset = 0;
		let buffer = std::sync::Mutex::new(SyncBuffer { buf, offset });
		let song = std::sync::RwLock::new(None);
		let likes = std::sync::RwLock::new(SyncLikes { songs: vec![], updated: std::time::SystemTime::UNIX_EPOCH });
		let (tx, rx) = std::sync::mpsc::channel();
		let working = Arc::new(std::sync::atomic::AtomicBool::new(false));

		let inner = Arc::new(SubsonicProviderInner { client, buffer, song, likes, tx, working: working.clone() });
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
		let mut buffer = self.0.buffer.lock().unwrap();

		if len > buffer.buf.len() {
			return None;
		}

		if buffer.offset + len > buffer.buf.len() {
			return None;
		}

		buffer.offset += len;
		let data = buffer.buf[buffer.offset-len..buffer.offset].to_vec();

		Some(data)
	}

	fn song(&self) -> Option<sunk::song::Song> {
		self.0.song.read().unwrap().clone()
	}

	fn likes(&self) -> Vec<sunk::song::Song> {
		if std::time::SystemTime::now() > self.0.likes.read().unwrap().updated + std::time::Duration::from_secs(300) {
			self.0.tx.send(Op::RefreshLikes);
		}
		
		self.0.likes.read().unwrap().songs.clone()
	}

	fn progress(&self) -> f32 {
		if self.0.song.read().unwrap().is_none() { return 0. }
		let buf = self.0.buffer.lock().unwrap();
		(buf.offset as f32 / buf.buf.len() as f32)
			.min(1.)
			.max(0.)
	}

	fn play(&self, song: sunk::song::Song) {
		self.0.tx.send(Op::PlaySong(song));
	}

	fn working(&self) -> bool {
		self.0.working.load(std::sync::atomic::Ordering::Relaxed)
	}
}

enum Op {
	PlaySong(sunk::song::Song),
	RefreshLikes,
}


fn work(ctx: Arc<SubsonicProviderInner>, rx: std::sync::mpsc::Receiver<Op>, working: Arc<std::sync::atomic::AtomicBool>) {
	while let Ok(op) = rx.recv() {
		working.store(true, std::sync::atomic::Ordering::Relaxed);
		match op {
			Op::PlaySong(mut song) => {

				song.set_max_bit_rate(256); // TODO make configurable
				song.set_transcoding("mp3");
	
				let mut temp = Vec::new();
				if match song.stream(&ctx.client) {
					Ok(mut reader) => match reader.read_to_end(&mut temp) {
						Ok(_) => true,
						Err(e) => {
							log::error!("error copying streamed data: {e}");
							false
						},
					},
					Err(e) => {
						log::error!("error requesting song stream: {e}");
						false
					},
				} {
					*ctx.song.write().unwrap() = Some(song.clone());
	
					let mut decoder = rmp3::Decoder::new(&temp);
					let mut out = Vec::new();
					while let Some(frame) = decoder.next() {
						match frame {
							rmp3::Frame::Audio(audio) => {
								// TODO is there a push_all??
								for sample in audio.samples() {
									out.push(*sample);
								}
							},
							rmp3::Frame::Other(_) => {},
						}
					}

					let mut buffer = ctx.buffer.lock().unwrap();
					buffer.offset = 0;
					buffer.buf = out;
				}
			},
			Op::RefreshLikes => {
				match ctx.client.starred(1) {
					Ok(res) => {
						let mut guard = ctx.likes.write().unwrap();
						guard.songs = res.songs;
						guard.updated = std::time::SystemTime::now();
					},
					Err(e) => log::error!("error fetching likes: {e}"),
				}
			},
		}
		working.store(false, std::sync::atomic::Ordering::Relaxed);
	}
	log::warn!("closing worker loop");
}

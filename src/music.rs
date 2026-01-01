use std::sync::Arc;

use sunk::{Streamable, song::Song};

pub trait MusicProvider<T>: Send + Clone {
	fn data(&self, len: usize) -> Option<Vec<T>>;
	fn wait(&self);
}



#[derive(Clone)]
pub struct SubsonicProvider(Arc<SubsonicProviderInner>);
struct SubsonicProviderInner {
	client: sunk::Client,
	buffer: std::sync::Mutex<SyncBuffer>,
	tx: std::sync::mpsc::Sender<()>,
	rx: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
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
		let (tx, rx) = std::sync::mpsc::channel();

		Ok(Self(Arc::new(SubsonicProviderInner { client, buffer, tx, rx: std::sync::Mutex::new(rx) })))
	}

	pub fn random_song(&mut self) -> Result<Option<Song>, sunk::Error> {
		let mut buffer = self.0.buffer.lock().unwrap();
		let songs = Song::random(&self.0.client, 1)?.into_iter().next();
		if let Some(mut song) = songs {
			song.set_max_bit_rate(320); // TODO make configurable
			song.set_transcoding("mp3");
			let mut temp = Vec::new();
			song.stream(&self.0.client)?.read_to_end(&mut temp).unwrap();
			buffer.offset = 0;
			buffer.buf = Vec::new();
			let mut decoder = rmp3::Decoder::new(&temp);
			while let Some(frame) = decoder.next() {
				match frame {
					rmp3::Frame::Audio(audio) => {
						// TODO is there a push_all??
						for sample in audio.samples() {
							buffer.buf.push(*sample);
						}
					},
					rmp3::Frame::Other(_) => {},
				}
			}
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
			self.0.tx.send(());
			return None;
		}

		buffer.offset += len;
		let data = buffer.buf[buffer.offset-len..buffer.offset].to_vec();

		Some(data)
	}

	fn wait(&self) {
		self.0.rx.lock().unwrap().recv();
	}
}


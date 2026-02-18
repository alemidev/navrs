use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::ext::err::IgnorableError;

#[derive(Debug, thiserror::Error)]
pub enum AudioSinkError {
	#[error("stream config error: {0} - {0:?}")]
	DefaultStreamConfig(#[from] cpal::DefaultStreamConfigError),

	#[error("error building stream: {0} - {0:?}")]
	BuildStream(#[from] cpal::BuildStreamError),

	#[error("play stream error: {0} - {0:?}")]
	PlayStream(#[from] cpal::PlayStreamError),

	#[error("supported configs error: {0} - {0:?}")]
	SupportedStreamConfigsError(#[from] cpal::SupportedStreamConfigsError),

	#[error("requested sample rate {0} is not supported by host")]
	UnsupportedSampleRate(u32),
}

#[derive(Clone)]
pub struct AudioPlayer {
	pub buffer: crate::ext::atomic::BufferHandle<f32>,
	pub paused: crate::ext::atomic::Flag,
	dev: Arc<cpal::Device>,
	stream: Arc<std::sync::Mutex<Option<cpal::Stream>>>,
	store: Arc<crate::ext::atomic::BufferHolder<f32>>,
}

impl AudioPlayer {
	pub fn new() -> Self {
		let paused = crate::ext::atomic::Flag::new(false);
		let host = cpal::default_host();
		let dev = Arc::new(host.default_output_device().expect("no output device available")); // TODO allow picking?

		let (buf_tx, buf_rx) = crate::ext::atomic::buffer();

		AudioPlayer {
			buffer: buf_tx,
			paused,
			dev,
			stream: Arc::new(std::sync::Mutex::new(None)),
			store: Arc::new(buf_rx),
		}
	}

	pub fn play(&self, data: Vec<f32>, sample_rate: u32) -> Result<(), AudioSinkError> {
		self.stream.lock().expect("mutex poisoned").iter().for_each(|s| s.pause().ignore());

		let cfgs = self.dev.supported_output_configs()?;

		let mut acceptable = false;
		for cfg in cfgs {
			if cfg.min_sample_rate() <= sample_rate && sample_rate <= cfg.max_sample_rate() {
				acceptable = true;
				break;
			}
		}
		if !acceptable {
			return Err(AudioSinkError::UnsupportedSampleRate(sample_rate));
		}

		let config = cpal::StreamConfig {
			sample_rate,
			channels: 2, // TODO can we assume this to be true??
			buffer_size: cpal::BufferSize::Default, // TODO do we want to change this?
		};

		self.buffer.seek(0);
		self.buffer.set(data);

		let paused = self.paused.clone();
		let store = self.store.clone();
		let mut stream = self.stream.lock().expect("mutex poisoned");
		let s = self.dev.build_output_stream(
			&config,
			move |data: &mut [f32], _info| {
				if paused.get() {
					return;
				}
				data.copy_from_slice(&store.read(data.len()));
			},
			|e| log::error!("error in stream callback: {e} - {e:?}"),
			None,
		)?;

		s.play().ignore();
		*stream = Some(s);

		Ok(())
	}

	pub fn clear(&self) {
		self.buffer.seek(0);
		self.buffer.set(Vec::new());
	}
}


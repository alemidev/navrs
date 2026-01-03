use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug, thiserror::Error)]
pub enum AudioSinkError {
	#[error("stream config error: {0}")]
	DefaultStreamConfig(#[from] cpal::DefaultStreamConfigError),

	#[error("error building stream: {0}")]
	BuildStream(#[from] cpal::BuildStreamError),

	#[error("play stream error: {0}")]
	PlayStream(#[from] cpal::PlayStreamError),
}

#[derive(Clone)]
pub struct AudioSink<T> {
	pub buffer: crate::ext::atomic::BufferHandle<T>,
	_stream: Arc<cpal::Stream>,
}

impl AudioSink<f32> {
	pub fn init(paused: crate::ext::atomic::Flag) -> Result<Self, AudioSinkError> {
		let host = cpal::default_host(); // TODO allow choosing host
		let dev = host.default_output_device().unwrap(); // TODO allow picking, and fallback
		let cfg = dev.default_output_config()?; // TODO allow changing config

		let (buf_tx, buf_rx) = crate::ext::atomic::buffer();

		let stream = dev.build_output_stream(
			&cfg.config(),
			move |data: &mut [f32], _info| {
				if paused.get() {
					return;
				}
				data.copy_from_slice(&buf_rx.read(data.len()));
			},
			|e| log::error!("error sending data to sink: {e}"), // TODO reset?
			None,
		)?;

		stream.play()?;

		Ok(Self { _stream: Arc::new(stream), buffer: buf_tx })
	}
}


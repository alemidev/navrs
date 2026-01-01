use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug, thiserror::Error)]
pub enum AudioSinkError {
	#[error("stream config error: {0}")]
	DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),

	#[error("error building stream: {0}")]
	BuildStreamError(#[from] cpal::BuildStreamError),
}

pub trait AudioController {
	fn play(&mut self);
	fn pause(&mut self);
}


pub struct AudioSink {
	stream: cpal::Stream,
}

impl AudioSink {
	pub fn init<T: crate::music::MusicProvider<f32> + 'static>(provider: T) -> Result<Self, AudioSinkError> {
		let host = cpal::default_host(); // TODO allow choosing host
		let dev = host.default_output_device().unwrap(); // TODO allow picking, and fallback
		let cfg = dev.default_output_config()?; // TODO allow changing config

		let stream = dev.build_output_stream(
			&cfg.config(),
			move |data: &mut [f32], _info| {
				match provider.data(data.len()) {
					Some(d) => data.copy_from_slice(&d),
					None => data.copy_from_slice(&vec![0f32; data.len()]),
				}
			},
			|e| log::error!("error sending data to sink: {e}"), // TODO reset?
			None,
		)?;

		Ok(Self { stream })
	}
}

impl AudioController for AudioSink {
	fn play(&mut self) {
		self.stream.play();
	}

	fn pause(&mut self) {
		self.stream.pause();
	}
}

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone)]
pub struct SongData {
	pub data: Vec<f32>,
	pub sample_rate: u32,
}


// a generic decoder implementation with symphonia
pub fn decode(data: &[u8]) -> Result<SongData, symphonia::core::errors::Error> {
	let buf = std::io::Cursor::new(data.to_vec());
	let mss = MediaSourceStream::new(Box::new(buf), Default::default());

	// Use the default options for metadata + format readers and decoder.
	let meta_opts: MetadataOptions = Default::default();
	let fmt_opts: FormatOptions = Default::default();
	let dec_opts: DecoderOptions = Default::default();

	// Probe the media source.
	let mut format = symphonia::default::get_probe()
		.format(&Hint::default(), mss, &fmt_opts, &meta_opts)?
		.format;

	// Find the first audio track with a known (decodeable) codec.
	let track = format
		.tracks()
		.iter()
		.find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
		.ok_or(symphonia::core::errors::end_of_stream_error::<Vec<f32>>().expect_err("ughh helper"))?;

	// Create a decoder for the track.
	let mut decoder = symphonia::default::get_codecs()
		.make(&track.codec_params, &dec_opts)?;

	let mut data = Vec::new();
	let mut sample_rate = 44100;

	// The decode loop.
	loop {
		let packet = match format.next_packet() {
			Ok(p) => p,
			Err(Error::IoError(_)) => break,
			Err(e) => {
				log::error!("error getting next packet: {e} - {e:?}");
				break;
			},
		};

		match decoder.decode(&packet) {
			Ok(audio_buf) => {
				sample_rate = audio_buf.spec().rate;
				let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec());
				sample_buf.copy_interleaved_ref(audio_buf);
				for s in sample_buf.samples() {
					data.push(*s);
				}
			}
			Err(Error::IoError(_)) => {
				// The packet failed to decode due to an IO error, skip the packet.
				continue;
			}
			Err(Error::DecodeError(_)) => {
				// The packet failed to decode due to invalid data, skip the packet.
				continue;
			}
			Err(_err) => {
				// An unrecoverable error occurred, halt decoding.
				log::warn!("finish decoding for error: {_err}");
				break;
			}
		}
	}

	Ok(SongData { data, sample_rate })
}


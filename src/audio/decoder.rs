use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;


pub fn decode(data: &[u8], ext: Option<String>) -> Result<Vec<f32>, symphonia::core::errors::Error> {
	let buf = std::io::Cursor::new(data.to_vec());
	let mss = MediaSourceStream::new(Box::new(buf), Default::default());

	let mut hint = Hint::new();

	// Create a probe hint using the file's extension. [Optional]
	if let Some(ext) = ext {
		hint.with_extension(&ext);
	}

	// Use the default options for metadata and format readers.
	let meta_opts: MetadataOptions = Default::default();
	let fmt_opts: FormatOptions = Default::default();

	// Probe the media source.
	let probed = symphonia::default::get_probe()
		.format(&hint, mss, &fmt_opts, &meta_opts)?;

	// Get the instantiated format reader.
	let mut format = probed.format;

	// Find the first audio track with a known (decodeable) codec.
	let track = format
		.tracks()
		.iter()
		.find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
		.expect("no supported audio tracks"); // TODO make it a symphonia error

	// Use the default options for the decoder.
	let dec_opts: DecoderOptions = Default::default();

	// Create a decoder for the track.
	let mut decoder = symphonia::default::get_codecs()
		.make(&track.codec_params, &dec_opts)
		.expect("unsupported codec");

	// Store the track identifier, it will be used to filter packets.
	// let track_id = track.id;

	let mut buf = None;

	// The decode loop.
	loop {
		// Get the next packet from the media format.
		let packet = format.next_packet()?;

		// // If the packet does not belong to the selected track, skip over it.
		// if packet.track_id() != track_id {
		// 	continue;
		// }

		// Decode the packet into audio samples.
		match decoder.decode(&packet) {
			Ok(audio_buf) => {
				if buf.is_none() {
					buf = Some(SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec()));
				}

				if let Some(b) = &mut buf {
					b.copy_interleaved_ref(audio_buf);
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

	match buf {
		None => symphonia::core::errors::decode_error("no valid packets were decoded"),
		Some(b) => Ok(b.samples().to_vec()),
	}
}


pub fn decode_mp3(data: &[u8]) -> Vec<f32> {
	let mut decoder = rmp3::Decoder::new(data);
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
	out
}

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;


// a rather crude generic decoder implementation with symphonia
// this is mostly copied from examples, can probably be polished a lot
// TODO what do we do about sample rate? should we:
//      * resample here?
//      * try to reinit output sink? what if rate is unsupported??

pub fn decode(data: &[u8], ext: Option<String>) -> Result<(Vec<f32>, u32), symphonia::core::errors::Error> {
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
		.ok_or(symphonia::core::errors::end_of_stream_error::<Vec<f32>>().expect_err("ughh helper"))?;

	// Use the default options for the decoder.
	let dec_opts: DecoderOptions = Default::default();

	// Create a decoder for the track.
	let mut decoder = symphonia::default::get_codecs()
		.make(&track.codec_params, &dec_opts)?;

	// Store the track identifier, it will be used to filter packets.
	// let track_id = track.id;

	let mut buf = Vec::new();
	let mut sample_rate = 44100;

	// The decode loop.
	loop {
		// Get the next packet from the media format.
		let packet = match format.next_packet() {
			Ok(p) => p,
			// Err(Error::IoError(_)) => break,
			Err(e) => {
				log::error!("error getting next packet: {e} - {e:?}");
				break;
			},
		};

		// // If the packet does not belong to the selected track, skip over it.
		// if packet.track_id() != track_id {
		// 	continue;
		// }
		
		// Decode the packet into audio samples.
		match decoder.decode(&packet) {
			Ok(audio_buf) => {
				sample_rate = audio_buf.spec().rate;
				let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec());
				sample_buf.copy_interleaved_ref(audio_buf);
				for s in sample_buf.samples() {
					buf.push(*s);
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

	Ok((buf, sample_rate))
}


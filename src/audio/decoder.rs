use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoderOptions};
use symphonia::core::common::Limit;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

#[derive(Clone)]
pub struct SongData {
	pub data: Vec<f32>,
	pub sample_rate: u32,
	pub image: Option<SongImage>,
}

#[derive(Clone)]
pub struct SongImage {
	pub data: Vec<u8>,
	pub format: Option<image::ImageFormat>,
}


// a generic decoder implementation with symphonia
pub fn decode(data: &[u8]) -> Result<SongData, symphonia::core::errors::Error> {
	let buf = std::io::Cursor::new(data.to_vec());
	let mss = MediaSourceStream::new(Box::new(buf), Default::default());

	// Use the default options for metadata + format readers and decoder.
	let meta_opts: MetadataOptions = MetadataOptions::default().limit_visual_bytes(Limit::None);
	let fmt_opts: FormatOptions = Default::default();
	let dec_opts: AudioDecoderOptions = Default::default();

	// Probe the media source.
	let mut format = symphonia::default::get_probe()
		.probe(&Hint::default(), mss, fmt_opts, meta_opts)?;

	// Find the first audio track with a known (decodeable) codec.
	let track = format.default_track(symphonia::core::formats::TrackType::Audio)
		.ok_or(symphonia::core::errors::unsupported_error::<()>("no audio track").expect_err("how do i use this helper?"))?;

	// Create a decoder for the track.
	let mut decoder = symphonia::default::get_codecs()
		.make_audio_decoder(&track.codec_params.as_ref().and_then(|c| c.audio().cloned()).unwrap_or(AudioCodecParameters::default()), &dec_opts)?;

	let mut data = Vec::new();
	let mut sample_rate = 44100;
	let mut image = None;

	//let track_id = track.id as u64;

	// TODO this only works if there's ONE image per track, however "sorting" image type by
	// desirability is a mess...
	if let Some(rev) = format.metadata().current() {
		for v in rev.media.visuals.iter() {
			image =  Some(SongImage {
				data: v.data.to_vec(),
				format: image::ImageFormat::from_mime_type(v.media_type.as_ref().map(|x| x.to_string()).unwrap_or_default()),
			});
		}
		// per-track cover should overrule global one, if present
		for per_track in rev.per_track.iter() {
			for v in per_track.metadata.visuals.iter() {
				image =  Some(SongImage {
					data: v.data.to_vec(),
					format: image::ImageFormat::from_mime_type(v.media_type.as_ref().map(|x| x.to_string()).unwrap_or_default()),
				});
			}
		}
	}

	// The decode loop.
	loop {
		let packet = match format.next_packet() {
			Ok(Some(p)) => p,
			Ok(None) => break,
			Err(Error::IoError(e)) => {
				log::warn!("decoder loop done: {e} - {e:?}");
				break;
			},
			Err(e) => {
				log::error!("error getting next packet: {e} - {e:?}");
				break;
			},
		};

		//if packet.track_id() != track_id {
		//	continue
		//}

		match decoder.decode(&packet) {
			Ok(audio_buf) => {
				sample_rate = audio_buf.spec().rate();
				let mut tmp = Vec::new();
				tmp.resize(audio_buf.samples_interleaved(), f32::MIN);
				audio_buf.copy_to_slice_interleaved(&mut tmp);
				for s in tmp {
					data.push(s);
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

	Ok(SongData { data, sample_rate, image })
}



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

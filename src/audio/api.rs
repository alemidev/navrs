
pub trait Player {
	fn paused(&self) -> bool;
	fn set_paused(&self, val: bool);

	fn resume(&self) {
		self.set_paused(false);
	}

	fn pause(&self) {
		self.set_paused(true);
	}

	fn play_pause(&self) {
		self.set_paused(!self.paused());
	}
}

pub trait Buffer<T> {
	fn progress(&self) -> f32;
	fn seek(&self, pos: f32);

	#[allow(unused)]
	fn is_empty(&self) -> bool {
		self.progress() >= 1.
	}

	fn restart(&self) {
		self.seek(0.);
	}

	fn skip(&self, amount: f32) {
		self.seek(self.progress() + amount.clamp(-1., 1.));
	}
}

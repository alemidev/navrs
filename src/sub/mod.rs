pub mod cache;
pub mod provider;

pub use provider::Provider;

pub type Song = submarine::data::Child;
pub type Id = String;

// pub trait MusicProvider<T>: Send + Clone {
// 	fn data(&self, len: usize) -> Option<Vec<T>>; // TODO cloneless??
// 
// 	fn current_song(&self) -> Option<Song>;
// 	fn likes(&self) -> Vec<Song>;
// 
// 	fn progress(&self) -> f32;
// 	fn working(&self) -> bool;
// 	fn running(&self) -> bool;
// 
// 	fn set_pause(&self, state: bool);
// 	fn play(&self, song: Id);
// 	fn next(&self);
// 
// 	fn queue(&self) -> Vec<Song>;
// 	fn enqueue(&self, songs: Vec<Song>);
// 	fn play_next(&self, song: Song);
// 
// 	#[deprecated = "can't be really done without locking, need higher-level methods maybe"]
// 	fn pop_queue(&self, idx: usize) -> Option<Song>;
// 	fn clear_queue(&self);
// 	fn seek(&self, percentage: f32);
// 
// 	fn toggle(&self) {
// 		self.set_pause(!self.running());
// 	}
// 
// 	fn pause(&self) {
// 		self.set_pause(true);
// 	}
// 
// 	fn resume(&self) {
// 		self.set_pause(false);
// 	}
// 
// 	fn restart(&self) {
// 		self.seek(0.);
// 	}
// 
// 	fn skip(&self, amount: f32) {
// 		self.seek(
// 			self.progress() + amount.clamp(-1., 1.)
// 		);
// 	}
// 
// 	fn shuffle_liked(&self) {
// 		let mut queue = self.likes();
// 		queue.shuffle(&mut rand::rng());
// 		self.clear_queue();
// 		self.enqueue(queue);
// 		self.next();
// 	}
// }

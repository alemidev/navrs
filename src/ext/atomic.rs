use std::{collections::VecDeque, sync::Arc};

use tokio::sync::{Mutex, watch};

use crate::ext::err::IgnorableError;

#[derive(Debug, Clone, Default)]
pub struct Flag(Arc<std::sync::atomic::AtomicBool>);
impl Flag {
	pub fn new(val: bool) -> Self {
		Self(Arc::new(std::sync::atomic::AtomicBool::new(val)))
	}

	// TODO the orderings!!!! what do they mean??? need to reread the docs ughhh
	pub fn get(&self) -> bool {
		self.0.load(std::sync::atomic::Ordering::Relaxed)
	}
	pub fn set(&self, val: bool) {
		self.0.store(val, std::sync::atomic::Ordering::Relaxed);
	}
	pub fn toggle(&self) -> bool {
		let prev = self.get();
		self.set(!prev);
		!prev
	}
}



#[derive(Clone)]
pub struct Buffer<T>(Arc<Mutex<BufferInner<T>>>, watch::Receiver<(usize, usize, f32)>);
struct BufferInner<T> {
	data: Vec<T>,
	offset: usize,
	progress: watch::Sender<(usize, usize, f32)>,
}

impl<T: Clone> BufferInner<T> {
	fn inner_read(&mut self, n: usize) -> Vec<T> {
		self.offset += n;
		self.progress.send((self.offset, self.data.len(), self.offset as f32 / self.data.len() as f32)).ignore();
		self.data[self.offset - n .. self.offset].to_vec()
	}
}

impl<T: Clone> Buffer<T> {
	pub fn new(data: Vec<T>) -> Self {
		let (tx, rx) = watch::channel((0, 0, 0.));
		let inner = Arc::new(Mutex::new(BufferInner {
			data,
			offset: 0,
			progress: tx,
		}));

		Self(inner, rx)
	}

	#[allow(unused)]
	pub async fn read(&self, n: usize) -> Vec<T> {
		self.0.lock().await.inner_read(n)
	}

	pub fn try_read(&self, n: usize) -> Option<Vec<T>> {
		match self.0.try_lock() {
			Ok(mut guard) => Some(guard.inner_read(n)),
			Err(_) => None,
		}
	}

	pub fn progress(&self) -> f32 {
		self.1.borrow().2
	}

	pub fn len(&self) -> usize {
		self.1.borrow().1
	}

	pub fn offset(&self) -> usize {
		self.1.borrow().0
	}

	pub async fn seek(&self, pos: f32) {
		let mut guard = self.0.lock().await;
		guard.offset = (guard.data.iter().len() as f32 * pos) as usize;
		guard.progress.send((guard.offset, guard.data.len(), pos)).ignore();
	}

	pub async fn set(&self, data: Vec<T>) {
		let mut guard = self.0.lock().await;
		guard.offset = 0;
		guard.data = data;
		guard.progress.send((guard.offset, guard.data.len(), 0.)).ignore();
	}
}



#[derive(Clone)]
pub struct Queue<T>(Arc<Mutex<QueueInner<T>>>, watch::Receiver<Vec<T>>, watch::Receiver<Option<T>>);
struct QueueInner<T> {
	storage: VecDeque<T>,
	snapshot: watch::Sender<Vec<T>>,
	peek: watch::Sender<Option<T>>,
}

impl<T: Clone> QueueInner<T> {
	fn pop(&mut self, back: bool) -> Option<T> {
		let x = if back {
			self.storage.pop_back()
		} else {
			self.storage.pop_front()
		};
		self.refresh();
		x
	}

	fn push(&mut self, item: T, front: bool) {
		if front {
			self.storage.push_front(item);
		} else {
			self.storage.push_back(item);
		}
		self.refresh();
	}

	fn refresh(&self) {
		self.snapshot.send(self.storage.clone().into_iter().collect()).ignore();
		self.peek.send(self.storage.front().cloned()).ignore();
	}
}

impl<T: Clone> Queue<T> {
	pub fn snapshot(&self) -> Vec<T> {
		self.1.borrow().clone()
	}

	pub async fn pop(&self) -> Option<T> {
		self.0.lock().await.pop(false)
	}

	#[allow(unused)]
	pub async fn pop_back(&self) -> Option<T> {
		self.0.lock().await.pop(true)
	}

	pub fn peek(&self) -> Option<T> {
		self.2.borrow().clone()
	}

	#[allow(unused)]
	pub async fn peek_back(&self) -> Option<T> {
		self.0.lock().await.storage.back().cloned()
	}

	pub async fn push(&self, item: T) {
		self.0.lock().await.push(item, false);
	}

	pub async fn push_front(&self, item: T) {
		self.0.lock().await.push(item, true);
	}

	pub fn new(data: Vec<T>) -> Self {
		let (peek, peek_rx) = watch::channel(None);
		let (snapshot, snapshot_rx) = watch::channel(Vec::new());
		let inner = QueueInner {
			storage: VecDeque::from(data),
			peek,
			snapshot,
		};
		inner.refresh();


		Self(Arc::new(Mutex::new(inner)), snapshot_rx, peek_rx)
	}

	pub async fn remove(&self, idx: usize) -> Option<T> {
		let mut guard = self.0.lock().await;
		let res = guard.storage.remove(idx);
		guard.refresh();
		res
	}

	pub async fn clear(&self) {
		let mut guard = self.0.lock().await;
		guard.storage.clear();
		guard.refresh();
	}
}


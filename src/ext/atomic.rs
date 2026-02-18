use std::sync::Arc;

use crate::ext::err::IgnorableError;



#[derive(Clone)]
pub struct Sync<T> {
	setter: tokio::sync::watch::Sender<T>,
	_holder: tokio::sync::watch::Receiver<T>,
}

impl<T: Clone> Sync<T> {
	pub fn new(x: T) -> Self {
		let (tx, rx) = tokio::sync::watch::channel(x);
		Self { setter: tx, _holder: rx }
	}
	pub fn get(&self) -> T {
		self.setter.borrow().clone()
	}
	pub fn set(&self, val: T) {
		self.setter.send(val).ignore();
	}
	pub fn inspect<X>(&self, fun: impl FnOnce(&T) -> X) -> X {
		fun(&self.setter.borrow())
	}
}




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
	#[allow(unused)]
	pub fn toggle(&self) -> bool {
		let prev = self.get();
		self.set(!prev);
		!prev
	}
}



#[derive(Debug, Clone, Default)]
pub struct Index(Arc<std::sync::atomic::AtomicUsize>);
impl Index {
	pub fn new(val: usize) -> Self {
		Self(Arc::new(std::sync::atomic::AtomicUsize::new(val)))
	}

	// TODO the orderings!!!! what do they mean??? need to reread the docs ughhh
	pub fn get(&self) -> usize {
		self.0.load(std::sync::atomic::Ordering::Relaxed)
	}
	pub fn set(&self, val: usize) {
		self.0.store(val, std::sync::atomic::Ordering::Relaxed);
	}
	pub fn inc(&self, val: usize) -> usize {
		let mut prev = self.get();
		prev += val;
		self.set(prev);
		prev
	}
	pub fn dec(&self, val: usize) -> usize {
		let mut prev = self.get();
		prev -= val;
		self.set(prev);
		prev
	}
}

pub fn buffer<T>() -> (BufferHandle<T>, BufferHolder<T>) {
	let idx = Index::new(0);
	let (tx, rx) = tokio::sync::watch::channel(Vec::new());
	(
		BufferHandle { idx: idx.clone(), setter: tx },
		BufferHolder { idx, rx },
	)
}

#[derive(Clone)]
pub struct BufferHandle<T> {
	idx: Index,
	setter: tokio::sync::watch::Sender<Vec<T>>,
}

#[derive(Clone)]
pub struct BufferHolder<T> {
	idx: Index,
	rx: tokio::sync::watch::Receiver<Vec<T>>,
}

impl<T: Clone + Default> BufferHolder<T> {
	pub fn read(&self, n: usize) -> Vec<T> {
		let i = self.idx.get();
		let l = self.rx.borrow().len();

		// out of bounds! return flatline
		if i >= l || n >= l || i + n >= l {
			// TODO there could be extra leftover data? this way it gets discarded!
			self.idx.set(l);
			return vec![T::default(); n];
			
		}
		self.idx.set(i + n);
		self.rx.borrow()[i .. i+n].to_vec()
	}
}

impl<T: Clone> BufferHandle<T> {
	pub fn pos(&self) -> usize {
		self.idx.get()
	}

	pub fn len(&self) -> usize {
		self.setter.borrow().len()
	}

	pub fn seek(&self, pos: usize) {
		self.idx.set(pos);
	}

	pub fn set(&self, data: Vec<T>) {
		self.setter.send(data).ignore();
	}
}



#[derive(Clone)]
pub struct Queue<T>(Arc<QueueInner<T>>);
struct QueueInner<T> {
	position: Index,
	setter: tokio::sync::watch::Sender<Vec<T>>,
	_holder: tokio::sync::watch::Receiver<Vec<T>>,
}

impl<T: Clone> Queue<T> {
	pub fn new(queue: Vec<T>) -> Self {
		let (setter, _holder) = tokio::sync::watch::channel(queue);
		Self(Arc::new(QueueInner { setter, _holder, position: Index::new(0) }))
	}

	pub fn current(&self) -> Option<T> {
		self.0.setter.borrow().get(self.0.position.get()).cloned()
	}
	pub fn index(&self) -> usize {
		self.0.position.get()
	}
	pub fn set_index(&self, pos: usize) {
		self.0.position.set(pos);
	}
	pub fn len(&self) -> usize {
		self.0.setter.borrow().len()
	}

	pub fn advance(&self) {
		self.0.position.inc(1);
	}

	pub fn set(&self, queue: Vec<T>) {
		self.0.setter.send_replace(queue);
		self.0.position.set(0);
	}
	pub fn dequeue(&self, pos: usize) {
		if pos >= self.len() {
			return;
		}
		self.0.setter.send_modify(|x| { x.remove(pos); });
		if pos < self.0.position.get() {
			self.0.position.dec(1);
		}
	}

	pub fn enqueue_at(&self, pos: usize, val: T) {
		let idx = self.0.position.get();
		// TODO load-bearing .insert() ...
		self.0.setter.send_modify(|data| data.insert(pos.min(data.len()), val));
		if pos <= idx {
			self.0.position.set(idx + 1);
		}
	}
	pub fn enqueue_next(&self, val: T) {
		self.enqueue_at(self.index() + 1, val);
	}
	pub fn enqueue(&self, val: T) {
		self.enqueue_at(self.len(), val);
	}

	pub fn get(&self, pos: usize) -> Option<T> {
		self.0.setter.borrow().get(pos).cloned()
	}

	pub fn view(&self) -> Vec<T> {
		self.0.setter.borrow().clone()
	}
}

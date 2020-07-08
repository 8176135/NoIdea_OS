// #[derive(Debug)]
// pub struct Semaphore {
// 	count: usize,
// 	// sleepers: []
// }
//
// impl Semaphore {}
//
// type Mutex = Semaphore;

use alloc::collections::{BTreeSet, BTreeMap};
use spin::{Mutex, RwLock};
use crate::processes::Name;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicI32, Ordering};

pub type SemaphoreId = i64;

lazy_static! { // Using a spinlock semaphore to control my semaphore lol
	pub static ref SEMAPHORE_STORE: RwLock<BTreeMap<SemaphoreId, Semaphore>> =
		RwLock::new(BTreeMap::new());
}

#[derive(Debug)]
pub struct Semaphore {
	initial_count: i32,
	count: AtomicI32,
	queue: Mutex<BTreeSet<Name>>,
}

impl Semaphore {
	pub	fn new(count: i32) -> Semaphore {
		Semaphore {
			initial_count: count,
			count: AtomicI32::new(count),
			queue: Mutex::new(BTreeSet::new()),
		}
	}
	
	pub fn is_neutral(&self) -> bool {
		let count = self.count.load(Ordering::Relaxed);
		((self.initial_count < 0 && count > 0)
		|| (self.initial_count == count))
		&& self.queue.try_lock().unwrap().len() == 0 // And Nothing in wait queue
	}
	
	pub fn wait(&self) -> bool {
		if self.count.load(Ordering::Relaxed) <= 0 {
			false
		} else {
			// TODO: Still don't quite get what atomic orderings I need
			self.count.fetch_sub(1, Ordering::Relaxed);
			true
		}
	}
	
	pub fn add_to_wait_queue(&self, name: Name) {
		assert_eq!(self.queue.lock().insert(name), false, "Wait queue already has element");
	}
	
	pub fn check_and_pop_if_exists(&self, name: Name) -> bool {
		self.queue.lock().remove(&name)
	}
	
	/// Returns true if current number of users is >= 0
	pub fn signal(&self) -> bool {
		let old = self.count.fetch_add(1, Ordering::Relaxed);
		old + 1 >= 0
	}
}
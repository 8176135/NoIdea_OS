// #[derive(Debug)]
// pub struct Semaphore {
// 	count: usize,
// 	// sleepers: []
// }
//
// impl Semaphore {}
//
// type Mutex = Semaphore;

use alloc::collections::VecDeque;
use alloc::collections::BTreeMap;
use spin::{Mutex, RwLock};
use crate::processes::Pid;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicI32, Ordering};

pub type SemaphoreId = i64;

lazy_static! { // Using a spinlock semaphore to control my semaphore lol
	pub static ref SEMAPHORE_STORE: RwLock<BTreeMap<SemaphoreId, Semaphore>> =
		RwLock::new(BTreeMap::new());
}

#[derive(Debug)]
pub struct Semaphore {
	count: AtomicI32,
	queue: Mutex<VecDeque<Pid>>,
}

impl Semaphore {
	pub	fn new(count: i32) -> Semaphore {
		Semaphore {
			count: AtomicI32::new(count),
			queue: Mutex::new(VecDeque::new()),
		}
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
	
	pub fn add_to_wait_queue(&self, pid: Pid) {
		self.queue.lock().push_back(pid);
	}
}
use alloc::collections::VecDeque;
use spin::Mutex;
use spin::RwLock;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU32, Ordering};

pub type FifoKey = u32;

lazy_static! {
	pub static ref FIFO_POOL: RwLock<BTreeMap<FifoKey, Mutex<VecDeque<u8>>>> = create_fifo_pool();
}

fn create_fifo_pool() -> RwLock<BTreeMap<FifoKey, Mutex<VecDeque<u8>>>> {
	RwLock::new(BTreeMap::new())
}

pub fn get_available_fifo_key() -> FifoKey {
	static FIFO_KEY_COUNTER: AtomicU32 = AtomicU32::new(1000);
	
	FIFO_KEY_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// /// Simple First in First out IPC
// struct FifoIpc<T> {
// 	pub queue: Mutex<VecDeque<T>>
// }
//
// impl<T> FifoIpc<T> {
// 	fn a() {
// 		RwLock::
// 	}
// }
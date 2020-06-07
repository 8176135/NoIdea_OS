pub mod scheduling;
pub mod process;

pub use process::Process;
use spin::Mutex;
use alloc::vec::Vec;
use crate::processes::scheduling::Scheduler;
use crate::special_collections::{IncrementingPool, DynamicBitmap};

pub type Name = u64;
pub type Pid = u64;

pub struct ProcessesManager {
	processes_list: Mutex<Vec<Option<Process>>>,
	scheduler: Scheduler,
	pid_pool: Mutex<IncrementingPool>,
	name_registry: Mutex<DynamicBitmap>,
}

impl ProcessesManager {
	pub fn new() -> Self {
		Self {
			processes_list: Mutex::new(vec![None; 2]),
			scheduler: Scheduler::new(vec![(1, 10), (2, 5), (3, 5), (1, 5), (3 , 10), (4, 5)]),
			pid_pool: Mutex::new(IncrementingPool::new(1)),
			name_registry: Mutex::new(DynamicBitmap::new()),
		}
	}
	
	pub fn create_new_process() {
	
	}
}
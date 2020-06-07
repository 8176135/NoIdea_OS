use crate::println;
use crate::kernel::{PROCESSES, CURRENT_PROCESS};
use core::sync::atomic::Ordering;
use x86_64::instructions::interrupts::without_interrupts;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
	static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler {
	time: usize,
	/// (Index, Time)
	periodic_order: Vec<(usize, usize)>,
}

impl Scheduler {
	
	pub fn new(periodic_order: Vec<(usize, usize)>) -> Self {
		Scheduler {
			time: 0,
			periodic_order
		}
	}
	
	pub fn next_tick(&mut self) {
		self.time += 1;
		self.schedule_new_process();
	}
	
	pub fn yielded(&mut self) {
		PROCESSES.lock()[CURRENT_PROCESS.load(Ordering::Relaxed)].as_ref().unwrap()
			.set_process_status(ProcessStatus::Yielded);
		self.schedule_new_process();
	}
	
	fn schedule_new_process(&mut self) {
	
	}
}


pub fn check_schedule(stack_pointer: usize) -> usize {
	println!("{:x}", stack_pointer);
	if CURRENT_PROCESS.load(Ordering::Relaxed) == 0 {
		let p_lock = PROCESSES.lock();
		for item in &*p_lock {
			if let Some(item) = item.as_ref() {
				return item.get_stack_pos().as_u64() as usize;
			}
		}
	} else {
	
	}
	// println!("{:x}", stack_pointer);
	
	stack_pointer
}
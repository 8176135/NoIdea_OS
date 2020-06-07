use crate::println;
use core::sync::atomic::Ordering;
use x86_64::instructions::interrupts::without_interrupts;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::processes::process::ProcessStatus;
use alloc::collections::VecDeque;
use crate::processes::{Pid, Name};

#[derive(Debug, Default)]
pub struct Scheduler {
	pub time: usize,
	/// (Name, Time)
	pub periodic_order: Vec<(Name, usize)>,
	pub periodic_time: usize,
	pub periodic_index: usize,
	pub device_queue: VecDeque<Pid>,
	pub sporadic_queue: VecDeque<Pid>,
}

impl Scheduler {
	pub fn new(periodic_order: Vec<(Name, usize)>) -> Self {
		Scheduler {
			periodic_order,
			..Default::default()
		}
	}
	
	pub fn get_current_periodic_entry(&self) -> (u64, usize) {
		self.periodic_order[self.periodic_index]
	}
	
	// pub fn next_tick(&mut self) {
	// 	self.time += 1;
	// 	// self.schedule_new_process();
	// }
	//
	// pub fn get_tick(&self) {
	//
	// }
	
	// pub fn yielded(&mut self) {
	// 	PROCESSES.lock()[CURRENT_PROCESS.load(Ordering::Relaxed)].as_ref().unwrap()
	// 		.set_process_status(ProcessStatus::Yielded);
	// 	self.schedule_new_process();
	// }
	//
	// fn schedule_new_process(&mut self) {
	//
	// }
	//
	// pub fn get_periodic_order(&self) -> &[(usize, usize)]{
	// 	&self.periodic_order
	// }
}
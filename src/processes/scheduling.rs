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
	pub periodic_yielded: bool,
	pub device_queue: VecDeque<Pid>,
	pub sporadic_queue: VecDeque<Pid>,
}

impl Scheduler {
	pub fn new(periodic_order: Vec<(Name, usize)>) -> Self {
		Scheduler {
			periodic_time: periodic_order[0].1,
			periodic_order,
			..Default::default()
		}
	}
	
	/// Check if periodic_time is 0, if yes change to next periodic period, returning new process Name,
	/// otherwise, return `None`
	pub fn check_and_change_periodic(&mut self) -> (Name, bool) {
		if self.periodic_time != 0 {
			return (self.get_current_periodic_entry().0, false);
		}
		
		self.increment_periodic_index();
		let (name, time) = self.get_current_periodic_entry();
		self.periodic_time = time;
		self.periodic_yielded = false;
		(name, true)
	}
	
	fn increment_periodic_index(&mut self) {
		self.periodic_index = (self.periodic_index + 1) % self.periodic_order.len();
		println!("Incremented Index: {}", self.periodic_index);
	}
	
	pub fn get_current_periodic_entry(&self) -> (Name, usize) {
		self.periodic_order[self.periodic_index]
	}
}
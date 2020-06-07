mod scheduling;
pub mod process;

pub use process::{Process};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::vec;
use lazy_static::lazy_static;
use crate::eprintln;
use crate::processes::scheduling::Scheduler;
use crate::special_collections::{IncrementingPool, DynamicBitmap};
use crate::processes::process::ProcessStatus;
use alloc::collections::VecDeque;
use x86_64::VirtAddr;

lazy_static! {
	pub static ref PROCESS_MANAGER: Mutex<ProcessesManager> = Mutex::new(ProcessesManager::new());
}

pub type Name = u64;
pub type Pid = u64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SchedulingLevel {
	Device = 0,
	Periodic = 1,
	Sporadic = 2,
}

pub struct ProcessesManager {
	processes_list: Vec<Option<Process>>,
	scheduler: Scheduler,
	currently_executing_process: Pid,
	pid_pool: IncrementingPool,
	name_registry: DynamicBitmap,
}

impl ProcessesManager {
	pub fn new() -> Self {
		Self {
			processes_list: vec![None; 2],
			currently_executing_process: 0,
			scheduler: Scheduler::new(vec![(1, 10), (2, 5), (3, 5), (1, 5), (3, 10), (4, 5)]),
			pid_pool: IncrementingPool::new(1),
			name_registry: DynamicBitmap::new(),
		}
	}
	
	pub fn create_new_process(&mut self, level: SchedulingLevel, name: Name, arg: i32, program_start: extern "C" fn()) -> Result<(), ()> {
		match level {
			SchedulingLevel::Device => {}
			SchedulingLevel::Periodic => {
				if !self.name_registry.set_bit(name as usize) { return Err(()); } // If the name has already been taken
				// And panic if the name doesn't exist in the order, since then the process can never be executed
				assert!(!self.scheduler.periodic_order.iter().any(|&(index, _)| index == name))
			}
			SchedulingLevel::Sporadic => {}
		}
		let process = Process::new(self.pid_pool.get_free_elem(), level, name, arg, program_start);
		
		if process.get_idx() >= self.processes_list.len() {
			self.processes_list.resize(process.get_idx() + 1, None);
		}
		let out_pid = process.get_pid();
		if level == SchedulingLevel::Sporadic {
			self.scheduler.sporadic_queue.push_back(out_pid);
		}
		
		assert!(self.processes_list[process.get_idx()].is_none(), "PID of new process is not empty");
		self.processes_list[process.get_idx()] = Some(process);
		
		
		Ok(())
	}
	
	// pub fn get_scheduler(&mut self) -> &mut Scheduler {
	// 	&mut self.scheduler
	// }
	
	pub fn next_tick_preempt_process(&mut self) -> Option<VirtAddr> {
		self.scheduler.time += 1;
		let current_time = self.scheduler.time as u64;
		self.scheduler.device_queue.extend(self.processes_list.iter()
			.filter_map(|c| c.as_ref())
			.filter(|c| c.get_process_scheduling_level() == SchedulingLevel::Device)
			.filter(|c| current_time % c.get_name() == 0)
			.map(|c| c.get_pid()));
		
		if let Some(current_process) = self.get_current_process_mut() {
			assert_eq!(current_process.get_process_status(), ProcessStatus::Running, "Currently running process is not running??");
			match current_process.get_process_scheduling_level() {
				SchedulingLevel::Device => {
					eprintln!("DEVICE TAKING MORE THAN 1 TICK TO COMPLETE!");
				}
				SchedulingLevel::Periodic | SchedulingLevel::Sporadic => {
					if !self.switch_to_next_device() { // If no device is scheduled.
						self.scheduler.periodic_time -= 1;
						if self.scheduler.periodic_time == 0 {
							// NOTE: Do NOT use `current_process` here, since the borrow checker doesn't understand separate structs
							self.get_current_process_mut().unwrap().set_process_status(ProcessStatus::Scheduled);
							if !self.switch_to_next_periodic() { // If no process scheduled for the next time slot
								if !self.switch_to_next_sporadic() { // No processes in sporadic queue either
									self.currently_executing_process = 0; // Idle
								}
							}
						}
					}
				}
			}
		} else {
			if !self.switch_to_next_device() { // If no device is scheduled.
				if !self.switch_to_next_periodic() { // If no process scheduled for the next time slot
					if !self.switch_to_next_sporadic() { // No processes in sporadic queue either
						self.currently_executing_process = 0; // Keep Idling
					}
				}
			}
		}
		self.get_current_process_mut().map(|c| c.get_stack_pos())
	}
	
	fn switch_to_next_device(&mut self) -> bool {
		if let Some(new_process) = self.scheduler.device_queue.front()
			.map(|&c| c) // Copy the value early to workaround borrow checker limitation
			.map(|new_pid| self.get_process_mut_with_pid(new_pid)
				.expect("No entry with pid in process list, PID from device queue, possibly caused by duplicate entries")) {
			new_process.set_process_status(ProcessStatus::Running);
			self.currently_executing_process = new_process.get_pid();
			
			true
		} else {
			false
		}
	}
	
	fn switch_to_next_periodic(&mut self) -> bool {
		self.scheduler.periodic_index += 1;
		let (name, time) = self.scheduler.get_current_periodic_entry();
		self.scheduler.periodic_time = time;
		if self.name_registry.check_bit(name as usize) {
			let mut new_process = self.processes_list.iter_mut()
				.find(|c| c.map(|c| c.get_name() == name).unwrap_or(false))
				.expect("Failed to find process with name, even though registered")
				.as_mut().unwrap(); // This is just to unwrap the process option, which we already checked
			self.currently_executing_process = new_process.get_pid();
			new_process.set_process_status(ProcessStatus::Running);
			true
		} else {
			false
		}
	}
	
	fn switch_to_next_sporadic(&mut self) -> bool {
		if let Some(new_process) = self.scheduler.sporadic_queue.front()
			.map(|&c| c) // Copy the value early to workaround borrow checker limitation
			.map(|new_pid| self.get_process_mut_with_pid(new_pid)
				.expect("No entry with pid in process list, PID from sporadic queue, possibly caused by duplicate entries")) {
			new_process.set_process_status(ProcessStatus::Running);
			self.currently_executing_process = new_process.get_pid();
			true
		} else {
			false // No entries in sporadic queue
		}
	}
	
	fn get_process_mut_with_pid(&mut self, pid: Pid) -> Option<&mut Process> {
		self.processes_list.get_mut(pid as usize - 1).and_then(|c| c.as_mut())
	}
	
	fn get_current_process_mut(&mut self) -> Option<&mut Process> {
		if self.currently_executing_process == 0 {
			None
		} else {
			Some(self.get_process_mut_with_pid(self.currently_executing_process)
				.expect("Current process is None, someone forgot to change process number when terminating"))
		}
	}
}
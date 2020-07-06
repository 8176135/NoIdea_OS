mod scheduling;
pub mod process;

pub use process::{Process};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::vec;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use crate::{eprintln, println};
use crate::processes::scheduling::Scheduler;
use crate::special_collections::{IncrementingPool, DynamicBitmap};
use crate::processes::process::ProcessStatus;
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
	Idle = 3,
}

pub struct ProcessesManager {
	processes_list: Vec<Option<Process>>,
	idle_process: Process,
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
			idle_process: Process::idle(),
			scheduler: Scheduler::new(vec![(1, 10), (2, 5), (3, 5), (1, 5), (2, 10), (4, 2)]),
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
				assert!(self.scheduler.periodic_order.iter().any(|&(inside_name, _)| inside_name == name),
						"Name not inside Periodic order")
			}
			SchedulingLevel::Sporadic => {}
			SchedulingLevel::Idle => panic!("You can't just go create a Idle process"),
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
		let idx = process.get_idx();
		self.processes_list[idx] = Some(process);
		
		Ok(())
	}
	
	pub fn yield_current_process(&mut self, stack_p: VirtAddr) -> VirtAddr {
		let current_process = self.get_current_process_mut();
		current_process.set_stack_pos(stack_p);
		let current_pid = current_process.get_pid();
		
		match current_process.get_process_scheduling_level() {
			SchedulingLevel::Device => {
				assert_eq!(self.scheduler.device_queue.pop_front(), Some(current_pid),
						   "Currently executing device not in the front of device queue?");
				self.schedule_all_the_stuff(false);
			}
			SchedulingLevel::Periodic => {
				current_process.set_process_status(ProcessStatus::Yielded);
				self.scheduler.periodic_yielded = true;
				if !self.switch_to_sporadic() {
					self.switch_to_idle(false);
				}
			}
			SchedulingLevel::Sporadic => {
				current_process.set_process_status(ProcessStatus::Scheduled);
				self.scheduler.sporadic_queue.rotate_left(1);
				// Sporadic processes don't really yield if there is nothing else running
				self.switch_to_sporadic();
			}
			SchedulingLevel::Idle => panic!("Why is the idle process yielding?!?!")
		}
		
		self.get_current_process().get_stack_pos()
	}
	
	pub fn get_current_process_arg(&self) -> i32 {
		self.get_current_process().get_arg()
	}
	
	pub fn end_current_process(&mut self) -> VirtAddr {
		// TODO: Maybe wipe the stack to prevent other processes from snooping on the memory?
		
		// Only use current_process immutably here
		let current_process =
			self.processes_list[self.currently_executing_process as usize - 1].take()
				.expect("Current process doesn't exist");
		
		crate::memory::dealloc_stack(current_process.get_stack_bounds(),
									 &mut *crate::TEMP_MAPPER.lock().as_mut().unwrap(),
									 &mut *crate::FRAME_ALLOCATOR.lock());
		
		match current_process.get_process_scheduling_level() {
			SchedulingLevel::Device => {
				assert_eq!(self.scheduler.device_queue.pop_front(), Some(current_process.get_pid()),
						   "Currently executing device not in the front of device queue?");
				// Could theoretically be faster if we used a linked list. But in practice I doubt it,
				// as removing anything is unlikely in the first place, and linked list is not cache friendly
				self.scheduler.device_queue.retain(|&c| c != current_process.get_pid());
			}
			SchedulingLevel::Periodic => {
				self.name_registry.clear_bit(current_process.get_name() as usize);
			}
			SchedulingLevel::Sporadic => {
				assert_eq!(self.scheduler.sporadic_queue.pop_front(), Some(current_process.get_pid()),
						   "Currently executing sporadic not in the front of sporadic queue?");
			}
			SchedulingLevel::Idle => panic!("The idle processes can't just end!?")
		}
		
		self.pid_pool.return_elem(current_process.get_pid());
		// Technically we won't be running the idle function, just looping in terminate. But that's fine for now
		self.switch_to_idle(false); // Ehh... maybe do something so the next task can immediately pick up before timer tick
		self.schedule_all_the_stuff(true);
		self.get_current_process().get_stack_pos()
	}
	
	/// Return None when it wants to just continue with whatever we are doing
	pub fn next_tick_preempt_process(&mut self, stack_p: usize) -> Option<VirtAddr> {
		self.scheduler.time += 1;
		let current_time = self.scheduler.time as u64;
		self.scheduler.device_queue.extend(self.processes_list.iter()
			.filter_map(|c| c.as_ref())
			.filter(|c| c.get_process_scheduling_level() == SchedulingLevel::Device)
			.filter(|c| current_time % c.get_name() == 0)
			.map(|c| c.get_pid()));
		
		let current_process = self.get_current_process_mut();
		assert_eq!(current_process.get_process_status(), ProcessStatus::Running, "Currently running process {} is not running??", current_process.get_arg());
		// Save the new stack position even if we don't need to change (because it's easier this way)
		current_process.set_stack_pos(VirtAddr::new(stack_p as u64));
		match current_process.get_process_scheduling_level() {
			SchedulingLevel::Device => {
				eprintln!("DEVICE TAKING MORE THAN 1 TICK TO COMPLETE!");
			}
			SchedulingLevel::Periodic | SchedulingLevel::Sporadic | SchedulingLevel::Idle =>
				self.schedule_all_the_stuff(true)
		}
		
		Some(self.get_current_process().get_stack_pos())
	}
	
	fn schedule_all_the_stuff(&mut self, tick: bool) {
		if !self.switch_to_device() { // If no device is scheduled.
			if tick {
				self.scheduler.periodic_time -= 1;
			}
			if !self.switch_to_periodic() { // If no process scheduled for the next time slot
				if !self.switch_to_sporadic() { // No processes in sporadic queue either
					self.switch_to_idle(true);
				}
			}
		}
	}
	
	fn switch_to_idle(&mut self, set_scheduled: bool) {
		if set_scheduled {
			let current_process = self.get_current_process_mut();
			if current_process.get_process_status() == ProcessStatus::Running {
				current_process.set_process_status(ProcessStatus::Scheduled);
			}
		}
		self.currently_executing_process = 0; // Idle
		self.idle_process.set_process_status(ProcessStatus::Running);
	}
	
	fn switch_to_device(&mut self) -> bool {
		let old_process = self.currently_executing_process;
		if let Some(new_process) = self.scheduler.device_queue.front()
			.map(|&c| c) // Copy the value early to workaround borrow checker limitation
			.map(|new_pid| self.get_process_mut_with_pid(new_pid)
				.expect("No entry with pid in process list, PID from device queue, possibly caused by duplicate entries")) {
			new_process.set_process_status(ProcessStatus::Running);
			// Have to do this to get around borrow checker
			self.currently_executing_process = new_process.get_pid();
			assert_ne!(old_process, self.currently_executing_process);
			self.get_process_mut_with_pid(old_process)
				.expect("Old process doesn't exist")
				.set_process_status(ProcessStatus::Scheduled);
			true
		} else {
			false
		}
	}
	
	fn switch_to_periodic(&mut self) -> bool {
		// self.get_current_process_mut().unwrap().set_process_status(ProcessStatus::Scheduled);
		
		let previously_yielded = self.scheduler.periodic_yielded;
		let old_name = self.scheduler.get_current_periodic_entry().0;
		let (name, _next_periodic) = self.scheduler.check_and_change_periodic();
		if self.scheduler.periodic_yielded {
			return false;
		}
		
		if previously_yielded { // If the previous
			let yielded_process = self.get_process_with_name(old_name, true)
				.expect("Yielded process not found");
			assert_eq!(yielded_process.get_process_status(), ProcessStatus::Yielded,
					   "Previously yielded but not yielded process status, this might be correct (but rare) behaviour though");
			yielded_process.set_process_status(ProcessStatus::Scheduled);
		}
		
		// Just because _next_periodic is true doesn't mean process changed.
		
		let current_process = self.get_current_process_mut();
		
		// Basically if there is nothing that needs changing, return true.
		if current_process.get_process_scheduling_level() == SchedulingLevel::Periodic &&
			current_process.get_name() == name {
			return current_process.get_process_status() != ProcessStatus::Yielded; // Return false if yielded, but moved to next periodic?, let sporadic schedule
		}
		
		// Otherwise, figure out the replacement
		if self.name_registry.check_bit(name as usize) {
			self.get_current_process_mut().set_process_status(ProcessStatus::Scheduled);
			let new_process = self.get_process_with_name(name, true).expect("Process not found");
			
			assert_eq!(new_process.get_process_status(), ProcessStatus::Scheduled,
					   "New process was already running: {}", new_process.get_name());
			new_process.set_process_status(ProcessStatus::Running);
			self.currently_executing_process = new_process.get_pid();
			true
		} else {
			false
		}
	}
	
	fn get_process_with_name(&mut self, name: Name, periodic_only: bool) -> Option<&mut Process> {
		self.processes_list.iter_mut()
			.find(|c| c.as_ref()
				.map(|c| (!periodic_only || c.get_process_scheduling_level() == SchedulingLevel::Periodic)
					&& c.get_name() == name).unwrap_or(false))
			.map(|c| c.as_mut().unwrap() ) // This is just to unwrap the process option, which we already checked
	}
	
	fn switch_to_sporadic(&mut self) -> bool {
		if let Some(new_process) = self.scheduler.sporadic_queue.front()
			.map(|&c| c) // Copy the value early to workaround borrow checker limitation
			.map(|new_pid| self.get_process_mut_with_pid(new_pid)
				.expect("No entry with pid in process list, PID from sporadic queue, possibly caused by duplicate entries")) {
			new_process.set_process_status(ProcessStatus::Running);
			let new_process_pid = new_process.get_pid();
			if self.currently_executing_process != new_process_pid {
				let current_process = self.get_current_process_mut();
				if current_process.get_process_status() == ProcessStatus::Running {
					current_process.set_process_status(ProcessStatus::Scheduled);
				}
				self.currently_executing_process = new_process_pid;
			}
			true
		} else {
			false // No entries in sporadic queue
		}
	}
	
	fn get_process_mut_with_pid(&mut self, pid: Pid) -> Option<&mut Process> {
		if pid == 0 {
			Some(&mut self.idle_process)
		} else {
			self.processes_list.get_mut(pid as usize - 1).and_then(|c| c.as_mut())
		}
	}
	
	fn get_process_with_pid(&self, pid: Pid) -> Option<&Process> {
		self.processes_list.get(pid as usize - 1).and_then(|c| c.as_ref())
	}
	
	fn get_current_process_mut(&mut self) -> &mut Process {
		if self.currently_executing_process == 0 {
			&mut self.idle_process
		} else {
			self.get_process_mut_with_pid(self.currently_executing_process)
				.expect("Current process is None, someone forgot to change process number when terminating")
		}
	}
	
	fn get_current_process(&self) -> &Process {
		if self.currently_executing_process == 0 {
			&self.idle_process
		} else {
			self.get_process_with_pid(self.currently_executing_process)
				.expect(&format!("Current process is None, someone forgot to change process number when terminating {}",
								 self.currently_executing_process))
		}
	}
	
	pub fn get_current_process_pid(&self) -> Pid {
		self.currently_executing_process
	}
}
use alloc::format;

pub extern "C" fn idle_process() -> ! {
	loop {
		x86_64::instructions::hlt(); // Save some (a lot of) power
	}
}
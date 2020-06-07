use x86_64::VirtAddr;
use super::Name;
use crate::kernel::os_terminate;
use crate::memory::{alloc_stack, StackBounds};
use crate::processes::{Pid, SchedulingLevel};
use crate::println;

global_asm!(include_str!("../setup_process_stack.s.out"));

extern "C" {
	fn asm_fake_register(new_stack_addr: usize, terminate_func_addr: usize, program_start_addr: usize) -> usize;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessStatus {
	Yielded = 0,
	Running = 1,
	Scheduled = 2,
}


#[derive(Clone, Debug)]
pub struct Process {
	pid: Pid,
	// paging_4_address: PhysAddr,
	level: SchedulingLevel,
	status: ProcessStatus,
	name: Name,
	// TODO: Replace this with pages
	stack_bounds: StackBounds,
	stack_pointer: VirtAddr,
	arg: i32,
}

impl Process {
	// TODO: Implement error type
	pub fn new(pid: Pid, level: SchedulingLevel, name: Name, arg: i32, program_start: extern "C" fn()) -> Process {
		let stack_bounds = alloc_stack(32, &mut *crate::TEMP_MAPPER.lock().as_mut().unwrap(),
									   &mut *crate::FRAME_ALLOCATOR.lock()).unwrap();
		
		let terminate_ret_addr = os_terminate as *const () as usize;
		stack_bounds.end();
		println!("Function address: {:x}", program_start as *const () as usize);
		let fake_int_sp = x86_64::instructions::interrupts::without_interrupts(|| {
			unsafe {
				asm_fake_register(stack_bounds.end().as_u64() as usize,
								  terminate_ret_addr,
								  program_start as *const () as usize)
			}
		});
		
		println!("Fake stack point: {:x}", fake_int_sp);
		Process {
			pid,
			level,
			status: ProcessStatus::Scheduled,
			stack_bounds,
			stack_pointer: VirtAddr::new(fake_int_sp as u64),
			name,
			arg,
		}
	}
	
	pub fn get_idx(&self) -> usize {
		self.pid as usize - 1
	}
	
	pub fn get_pid(&self) -> Pid {
		self.pid
	}
	
	pub fn get_name(&self) -> Name {
		self.name
	}
	
	pub fn get_stack_pos(&self) -> VirtAddr {
		self.stack_pointer
	}
	
	pub fn get_process_scheduling_level(&self) -> SchedulingLevel {
		self.level
	}
	
	pub fn set_process_status(&mut self, new_status: ProcessStatus) {
		self.status = new_status;
	}
	
	pub fn get_process_status(&self) -> ProcessStatus {
		self.status
	}
}
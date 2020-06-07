use x86_64::VirtAddr;
use super::Name;
use crate::memory::{alloc_stack, StackBounds};
use crate::processes::Pid;

global_asm!(include_str!("setup_process_stack.s.out"));

extern "C" {
	fn asm_fake_register(new_stack_addr: usize, terminate_func_addr: usize, program_start_addr: usize) -> usize;
}

#[derive(Copy, Clone, Debug)]
pub enum SchedulingLevel {
	Device = 0,
	Periodic = 1,
	Sporadic = 2,
}

#[derive(Copy, Clone, Debug)]
pub enum ProcessStatus {
	Yielded = 0,
	Running = 1,
	Scheduled = 2,
}


#[derive(Clone, Copy, Debug)]
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
	pub fn new(pid: Pid, level: SchedulingLevel, name: Name, arg: i32, program_start: extern "C" fn()) -> Result<Process, ()> {
		match level {
			SchedulingLevel::Device => {}
			SchedulingLevel::Periodic => {
				if !NAME_REGISTRY.lock().set_bit(name as usize) {
					// Name already taken
					return Err(());
				}
			}
			SchedulingLevel::Sporadic => {}
		}
		
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
		
		println!("{:x}", fake_int_sp);
		Ok(Process {
			pid,
			level,
			status: ProcessStatus::Scheduled,
			stack_bounds,
			stack_pointer: VirtAddr::new(fake_int_sp as u64),
			name,
			arg,
		})
	}
	
	fn get_idx(&self) -> usize {
		self.pid as usize - 1
	}
	
	pub fn get_stack_pos(&self) -> VirtAddr {
		self.stack_pointer
	}
	
	pub fn set_regs(&mut self, new_regs: Registers) {
		self.regs = Some(new_regs);
	}
	
	pub fn set_process_status(&mut self, new_status: ProcessStatus) {
	
	}
}
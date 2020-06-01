use crate::interrupts::interrupt_init;
use crate::gdt::gdt_init;
use crate::{println, print};
use crate::special_collections::{IncrementingPool, DynamicBitmap};
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::vec::Vec;

type Name = u64;
type Pid = u64;

#[derive(Clone, Copy, Debug)]
struct Registers {}

#[derive(Clone, Copy, Debug)]
struct Process {
	pid: u64,
	regs: Option<Registers>,
	level: SchedulingLevel,
	name: Name,
	arg: i32,
}

impl Process {
	// TODO: Implement error type
	fn new(level: SchedulingLevel, name: Name, arg: i32) -> Result<Process, ()> {
		match level {
			SchedulingLevel::Device => {},
			SchedulingLevel::Periodic => {
				if !NAME_REGISTRY.lock().set_bit(name as usize) {
					// Name already taken
					return Err(())
				}
			},
			SchedulingLevel::Sporadic => {},
		}
		let pid = PID_POOL.lock().get_free_elem();

		Ok(Process {
			pid,
			regs: None,
			level,
			name,
			arg,
		})
	}
}

lazy_static! {
	static ref PROCESSES: Mutex<Vec<Process>> = Mutex::new(Vec::new());
	static ref PID_POOL: Mutex<IncrementingPool> = Mutex::new(IncrementingPool::new(2));
	static ref NAME_REGISTRY: Mutex<DynamicBitmap> = Mutex::new(DynamicBitmap::new());
}

pub fn os_init() {
	interrupt_init();
	gdt_init();
}

pub fn os_start() {}

#[derive(Copy, Clone, Debug)]
enum SchedulingLevel {
	Device = 0,
	Periodic = 1,
	Sporadic = 2,
}

fn os_create<F: FnOnce() -> ()>(f: F, arg: i32, level: SchedulingLevel, name: i32) -> Result<u64, ()>{
	// Process::new(level, name, arg);

	// Process::new()
	unimplemented!()
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}
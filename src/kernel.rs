use crate::interrupts::interrupt_init;
use crate::gdt::gdt_init;
use crate::{println, print, static_bitmap, static_stack};
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::static_collections::bitmap::Bitmap;
use crate::static_collections::stack::Stack;

const MAX_PROCESS: usize = 16;

static_bitmap!(NameReg, NAME_REG, 16);
static_stack!(PidPool, PID_POOL, 16);

#[derive(Clone, Copy, Debug)]
struct Registers {}

#[derive(Clone, Copy, Debug)]
struct Process {
	pid: u64,
	regs: Option<Registers>,
	level: SchedulingLevel,
	name: u32,
	arg: i32,
}

impl Process {
	// TODO: Implement error type
	fn new(level: SchedulingLevel, name: u32, arg: i32) -> Result<Process, ()> {
		match level {
			SchedulingLevel::Device => {},
			SchedulingLevel::Periodic => {
				if !NAME_REG.lock().set_bit(name as usize).unwrap() {
					// Name already taken
					return Err(())
				}
			},
			SchedulingLevel::Sporadic => {},
		}
		let pid = PID_POOL.lock().pop().map_err(|e| ())?;
		
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
	static ref PROCESSES: Mutex<[Option<Process>; MAX_PROCESS]> = Mutex::new([None; MAX_PROCESS]);
}

pub fn os_init() {
	interrupt_init();
	gdt_init();
	{
		let mut pid_pool = PID_POOL.lock();
		for i in 0..16 {
			pid_pool.push(i);
		}
	}
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
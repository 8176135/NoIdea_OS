use crate::interrupts::interrupt_init;
use crate::gdt::gdt_init;
use crate::{println, print};
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_PROCESS: usize = 16;



#[derive(Clone, Copy, Debug)]
struct Registers {

}

#[derive(Clone, Copy, Debug)]
struct Process {
	pid: u64,
	regs: Option<Registers>,
	name: i32,
	arg: i32,
}

impl Process {
	const fn new(pid: u64, name: i32, arg: i32) -> Process {
		Process {
			pid,
			regs: None,
			name,
			arg
		}
	}
}

lazy_static! {
	static ref PROCESSES: Mutex<[Option<Process>; MAX_PROCESS]> = Mutex::new([None; MAX_PROCESS]);
}

pub fn os_init() {
	interrupt_init();
	gdt_init();
}

pub fn os_start() {

}

#[derive(Copy, Clone, Debug)]
enum SchedulingLevel {
	Device = 0,
	Periodic = 1,
	Sporadic = 2,
}

fn os_create<F: FnOnce() -> ()>(f: F, arg: i32, level: SchedulingLevel, name: i32) {
	// Process::new()
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}
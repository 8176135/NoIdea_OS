use crate::interrupts::interrupt_init;
use crate::gdt::gdt_init;
use crate::{println, print, helper};
use crate::special_collections::{IncrementingPool, DynamicBitmap};
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::vec::Vec;
use alloc::vec;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::idt::InterruptStackFrame;
use crate::memory::{StackBounds, alloc_stack};
use volatile::Volatile;
use crate::kernel::SchedulingLevel::Periodic;
use x86_64::instructions::interrupts::without_interrupts;
use core::sync::atomic::AtomicUsize;
use crate::processes::process::SchedulingLevel::Periodic;

pub static CURRENT_PROCESS: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
	pub static ref PROCESSES: Mutex<Vec<Option<Process>>> = Mutex::new(vec![None; 2]);
	static ref PID_POOL: Mutex<IncrementingPool> = Mutex::new(IncrementingPool::new(1));
	static ref NAME_REGISTRY: Mutex<DynamicBitmap> = Mutex::new(DynamicBitmap::new());
}

pub fn os_init() {
	interrupt_init();
	gdt_init();
}

#[inline(never)]
pub fn os_start() {
	// crate::interrupts::hardware::random_thing(123);
	// crate::context_switch::context_switch(123, Some(123));
	os_create(123, Periodic, 333, *&test_app);
	tester(1);
}

extern "C" fn os_terminate() -> ! {
	println!("We out of there!");
	loop {}
}

#[inline(never)]
fn tester(item: usize) {
	let other: usize;
	helper::print_stack_pointer();
	loop {}
}

fn os_create(arg: i32, level: SchedulingLevel, name: Name, f: extern "C" fn()) -> Result<u64, ()> {
	let process = Process::new(level, name, arg, f)?;
	
	Ok(without_interrupts(|| {
		let mut lock = PROCESSES.lock();
		
		if process.get_idx() >= lock.len() {
			lock.resize(process.get_idx() + 1, None);
		}
		let out_pid = process.pid;
		
		assert!(lock[process.get_idx()].is_none(), "PID of new process is not empty");
		lock[process.get_idx()] = Some(process);
		out_pid
	}))
}

extern "C" fn test_app() {
	println!("YOLO!!");
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}
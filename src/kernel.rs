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
use x86_64::instructions::interrupts::without_interrupts;
use core::sync::atomic::AtomicUsize;
use crate::processes::SchedulingLevel::Periodic;
use crate::processes::{SchedulingLevel, Name};
use crate::processes::PROCESS_MANAGER;

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

pub extern "C" fn os_terminate() -> ! {
	println!("We out of there!");
	loop {}
}

#[inline(never)]
fn tester(item: usize) {
	let other: usize;
	helper::print_stack_pointer();
	loop {}
}

fn os_create(arg: i32, level: SchedulingLevel, name: Name, f: extern "C" fn()) -> Result<(), ()> {
	let p_manager_lock = PROCESS_MANAGER.lock();
	Ok(())
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
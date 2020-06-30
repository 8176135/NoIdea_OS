use crate::interrupts::{interrupt_init, syscall1, SyscallCommand, syscall_handler};
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
use x86_64::registers::rflags::RFlags;
use core::intrinsics::breakpoint;
use core::ops::Deref;

pub fn os_init() {
	interrupt_init();
	gdt_init();
	
	use x86_64::registers::model_specific::{LStar, SFMask, KernelGsBase, Star, Efer, EferFlags, GsBase};
	// Store syscall location
	
	LStar::write(VirtAddr::new(syscall_handler as u64));
	SFMask::write(RFlags::INTERRUPT_FLAG & RFlags::TRAP_FLAG);
	let num_to_save = crate::gdt::TSS.deref() as *const _ as u64;
	KernelGsBase::write(VirtAddr::new(num_to_save));
	
	// FIXME: IDK WHATS GOING ON!!!
	// I don't understand why GsBase needs to be set, but otherwise `swapgs` doesn't work properly
	// Or maybe it does work properly.
	// GsBase::write(VirtAddr::new(num_to_save));
	
	unsafe {
		Star::write_raw(0, crate::gdt::GDT.1.code_selector.0);
		Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
	}
	
	x86_64::instructions::interrupts::enable();
}

#[naked]
pub unsafe extern "C" fn stuff() {
	let mut a = 0;
	a += 1;
	// println!("{}", a);
}

#[inline(never)]
pub fn os_start() {
	// crate::interrupts::hardware::random_thing(123);
	// crate::context_switch::context_switch(123, Some(123));
	// println!("{:?}", x86_64::registers::model_specific::KernelGsBase::read());
	// println!("{:?}", x86_64::registers::model_specific::GsBase::read());
	syscall1(SyscallCommand::Yield);
	// os_create(123, Periodic, 333, *&test_app);
	// tester(1);
}

pub extern "C" fn os_terminate() -> ! {
	println!("We out of there!");
	PROCESS_MANAGER.lock().end_current_process();
	
	loop {} // Wait for the next tick (Probably could yield or something here)
}

fn os_yield() {
	PROCESS_MANAGER.lock().yield_current_process();
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
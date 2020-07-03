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
	SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::TRAP_FLAG);
	KernelGsBase::write(VirtAddr::new(crate::gdt::TSS.deref() as *const _ as u64));
	
	
	// FIXME: IDK WHATS GOING ON!!!
	// I don't understand why GsBase needs to be set, but otherwise `swapgs` doesn't work properly
	// Or maybe it does work properly.
	// GsBase::write(VirtAddr::new(crate::gdt::TSS.deref() as *const _ as u64));
	
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
	// syscall1(SyscallCommand::Yield);
	println!("test_app: {:x}", test_app as u64);
	os_create(123, SchedulingLevel::Periodic, 4, *&test_app).unwrap();
	os_create(234, SchedulingLevel::Periodic, 3, *&test_app).unwrap();
	os_create(345, SchedulingLevel::Periodic, 2, *&test_app).unwrap();
	os_create(456, SchedulingLevel::Periodic, 1, *&test_app).unwrap();
	os_create(9990, SchedulingLevel::Sporadic, 1, *&test_app_spor).unwrap();
	os_create(9991, SchedulingLevel::Sporadic, 2, *&test_app_spor).unwrap();
	os_create(9992, SchedulingLevel::Sporadic, 3, *&test_app_spor).unwrap();
	os_create(9993, SchedulingLevel::Sporadic, 4, *&test_app_spor).unwrap();
	os_create(415, SchedulingLevel::Device, 10, *&test_app_device).unwrap();
	os_create(416, SchedulingLevel::Device, 15, *&test_app_device).unwrap();
	// os_create(456, Periodic, 1, *&test_app).unwrap();
	// tester(1);
}

pub extern "C" fn os_terminate() {
	println!("We out of there!");
	syscall1(SyscallCommand::Terminate);
}

pub fn os_yield() {
	syscall1(SyscallCommand::Yield);
}

pub fn os_getparam() -> i32 {
	PROCESS_MANAGER.lock().get_current_process_arg()
}

// #[inline(never)]
// fn tester(item: usize) {
// 	let other: usize;
// 	helper::print_stack_pointer();
// 	loop {}
// }

fn os_create(arg: i32, level: SchedulingLevel, name: Name, f: extern "C" fn()) -> Result<(), ()> {
	// No need to turn off interrupts because we lock process_manager
	let mut p_manager_lock = PROCESS_MANAGER.lock();
	p_manager_lock.create_new_process(level, name, arg, f)
	// Ok(())
}

extern "C" fn test_app() {
	let mut a: i64 = 0;
	for i in 0..5000000 {
		a.wrapping_add(i);
	}
	println!("Enter the gates: {} {}", os_getparam(), a);
	os_yield();
	println!("YOLO!! {}", interrupts::are_enabled());
}

extern "C" fn test_app_spor() {
	let mut a: i64 = 0;
	let param = os_getparam();
	for i in 0..10000000 {
		if i % 1000000 == 0 {
			println!("SPORE!!! {}", param);
			os_yield();
		}
		a.wrapping_add(i);
	}
	os_create(param + 4, SchedulingLevel::Sporadic, 123, test_app_spor);
	println!("WE OUT!! {}", param);
}

extern "C" fn test_app_device() {
	for i in 0..200 {
		println!("Reminder for stuff from {} {} ", os_getparam(), i);
		os_yield();
	}
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}
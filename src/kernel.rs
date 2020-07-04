use crate::interrupts::{interrupt_init, syscall1, SyscallCommand, syscall_handler};
use crate::gdt::gdt_init;
use crate::{println, print, helper};
use crate::special_collections::{IncrementingPool, DynamicBitmap};
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::prelude::v1::*;
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
use crate::ipc::FifoKey;

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
	// println!("test_app: {:x}", test_app as u64);
	os_create(123, SchedulingLevel::Periodic, 4, write_test_app);
	// os_create(123, SchedulingLevel::Periodic, 4, test_app).unwrap();
	// os_create(234, SchedulingLevel::Periodic, 3, test_app).unwrap();
	// os_create(345, SchedulingLevel::Periodic, 2, test_app).unwrap();
	// os_create(456, SchedulingLevel::Periodic, 1, test_app).unwrap();
	// os_create(9990, SchedulingLevel::Sporadic, 1,test_app_spor).unwrap();
	// os_create(9991, SchedulingLevel::Sporadic, 2,test_app_spor).unwrap();
	// os_create(9992, SchedulingLevel::Sporadic, 3,test_app_spor).unwrap();
	// os_create(9993, SchedulingLevel::Sporadic, 4,test_app_spor).unwrap();
	// os_create(10, SchedulingLevel::Device, 10, test_app_device).unwrap();
	// os_create(15, SchedulingLevel::Device, 15, test_app_device).unwrap();
}

pub extern "C" fn os_terminate() {
	// println!("We out of there!");
	syscall1(SyscallCommand::Terminate);
}

pub fn os_yield() {
	syscall1(SyscallCommand::Yield);
}

pub fn os_getparam() -> i32 {
	PROCESS_MANAGER.lock().get_current_process_arg()
}

fn os_create(arg: i32, level: SchedulingLevel, name: Name, f: extern "C" fn()) -> Result<(), ()> {
	// No need to turn off interrupts because we lock process_manager
	let mut p_manager_lock = PROCESS_MANAGER.lock();
	p_manager_lock.create_new_process(level, name, arg, f)
	// Ok(())
}

fn os_init_fifo() -> FifoKey {
	use crate::ipc;
	use alloc::collections::VecDeque;
	
	let mut fifo_pool = ipc::FIFO_POOL.write();
	let key = ipc::get_available_fifo_key();
	fifo_pool.insert(key, Mutex::new(VecDeque::new()));
	
	key
}

/// Write data to key
///
/// Returns Ok if written, otherwise TODO: error type
fn os_write(key: FifoKey, data: &[u8]) -> Result<(), ()> {
	use crate::ipc;
	// TODO: Check for ownership
	let fifo_pool = ipc::FIFO_POOL.read();
	let first = fifo_pool.get(&key).ok_or(())?;
	first.lock().extend(data.iter().cloned());
	
	Ok(())
}

/// Read into buf TODO: error type
fn os_read(key: FifoKey, buf: &mut [u8]) -> Result<usize, ()> {
	use crate::ipc;
	// TODO: Check for ownership
	let fifo_pool = ipc::FIFO_POOL.read();
	let first = fifo_pool.get(&key).ok_or(())?;
	for (idx, data) in buf.iter_mut().enumerate() {
		if let Some(c) = first.lock().pop_front() {
			*data = c
		} else {
			return Ok(idx)
		}
	}
	
	Ok(buf.len())
}

extern "C" fn test_app() {
	let mut a: i64 = 0;
	let mut stuff: Vec<u8> = Vec::new();
	for i in 0..5000000 {
		a.wrapping_add(i);
	}
	
	println!("Enter the gates: {} {}", os_getparam(), a);
	stuff.resize(20000, 0);
	println!("YOLO!! {}", interrupts::are_enabled());
}

extern "C" fn write_test_app() {
	let stuff = "Yep this works".to_owned();
	
	let fifo_stuff = os_init_fifo();
	os_create(fifo_stuff as i32, SchedulingLevel::Sporadic, 10, read_test_app);
	
	os_write(fifo_stuff, &postcard::to_allocvec(&stuff).unwrap());
	os_write(fifo_stuff, &postcard::to_allocvec(&"STUFF".to_owned()).unwrap());
	os_write(fifo_stuff, &postcard::to_allocvec(&stuff).unwrap());
	println!("Write!! {}", os_getparam());
}

extern "C" fn read_test_app() {
	let fifo_stuff = os_getparam() as u32;
	let mut stuff_output = [0u8; 40];
	let length_read = os_read(fifo_stuff, &mut stuff_output).unwrap();
	let (out, remaining): (String, _) =
		postcard::take_from_bytes(&stuff_output).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	let (out, remaining): (String, _) =
		postcard::take_from_bytes(remaining).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	let (out, remaining): (String, _) =
		postcard::take_from_bytes(remaining).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	println!("Read!! {}", length_read);
}

extern "C" fn test_app_spor() {
	let mut a: i64 = 0;
	let param = os_getparam();
	for i in 0..10000000 {
		if i % 10000000 == 0 {
			// println!("SPORE!!! {}", param);
			os_yield();
		}
		a.wrapping_add(i);
	}
	os_create(param + 4, SchedulingLevel::Sporadic, 123, test_app_spor);
	println!("WE OUT!! {}", param);
}

extern "C" fn test_app_device() {
	for i in 0..50 {
		println!("Reminder for stuff from {} {} ", os_getparam(), i);
		os_yield();
	}
	os_create(os_getparam() + 10, SchedulingLevel::Device, (os_getparam() + 10) as u64, test_app_device).unwrap();
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}
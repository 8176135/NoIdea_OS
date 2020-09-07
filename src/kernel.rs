#![allow(dead_code)]

use crate::interrupts::{interrupt_init, syscall1, SyscallCommand, syscall_handler};
use crate::gdt::gdt_init;
use crate::{println, eprintln};
use x86_64::instructions::interrupts;
use spin::Mutex;

use alloc::prelude::v1::*;
use x86_64::VirtAddr;
use crate::processes::{SchedulingLevel, Name};
use crate::processes::PROCESS_MANAGER;
use x86_64::registers::rflags::RFlags;
use core::ops::Deref;
use crate::ipc::FifoKey;
use crate::sync::{SemaphoreId, SEMAPHORE_STORE, Semaphore};
use x86_64::instructions::interrupts::{without_interrupts,
									   disable as disable_int,
									   enable as enable_int};

pub fn os_init() {
	interrupt_init();
	
	gdt_init();
	
	use x86_64::registers::model_specific::{LStar, SFMask, KernelGsBase, Star, Efer, EferFlags};
	// Store syscall location
	
	LStar::write(VirtAddr::new(syscall_handler as u64));
	SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::TRAP_FLAG);
	KernelGsBase::write(VirtAddr::new(crate::gdt::TSS.deref() as *const _ as u64));
	
	unsafe {
		Star::write_raw(0, crate::gdt::GDT.1.code_selector.0);
		Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
	}
}

#[inline(never)]
pub fn os_start() {
	// println!("test_app: {:x}", test_app as u64);
	os_create(123, SchedulingLevel::Sporadic, 1,
			  crate::tests::app_test_runner::run_tests).unwrap();
	// os_create(123, SchedulingLevel::Periodic, 1, test_app_signals).unwrap();
	// os_create(123, SchedulingLevel::Sporadic, 4, test_app_signals_recv).unwrap();
	// os_create(123, SchedulingLevel::Periodic, 4, write_test_app).unwrap();
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

pub(crate) fn os_create(arg: i32, level: SchedulingLevel, name: Name, f: extern "C" fn()) -> Result<(), ()> {
	// No need to turn off interrupts because we lock process_manager
	let mut p_manager_lock = PROCESS_MANAGER.lock();
	p_manager_lock.create_new_process(level, name, arg, f)
	// Ok(())
}

pub fn os_init_fifo() -> FifoKey {
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
pub fn os_write(key: FifoKey, data: &[u8]) -> Result<(), ()> {
	use crate::ipc;
	// TODO: Check for ownership
	let fifo_pool = ipc::FIFO_POOL.read();
	let first = fifo_pool.get(&key).ok_or(())?;
	first.lock().extend(data.iter().cloned());
	
	Ok(())
}

/// Read into buf TODO: error type
pub fn os_read(key: FifoKey, buf: &mut [u8]) -> Result<usize, ()> {
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

// Returns error if semaphore already exists
pub fn os_init_sem(id: SemaphoreId, initial_count: i32) -> Result<(), ()> {
	// TODO: A syscall probably should happen as normal processes wouldn't
	// be allowed to directly lock the semaphore store
	without_interrupts(|| {
		let mut store =
			SEMAPHORE_STORE.try_write().expect("DEADLOCK");
		if store.contains_key(&id) {
			Err(())
		} else {
			store.insert(id, Semaphore::new(initial_count));
			Ok(())
		}
	})
}

// Drop semaphore TODO: Added proper error type.
pub fn os_drop_sem(id: SemaphoreId) -> Result<(), &'static str> {
	without_interrupts(|| {
		let mut store = SEMAPHORE_STORE.try_write().expect("DEADLOCK");
		let out = store.remove(&id).ok_or("Semaphore Doesn't exist")?;
		if out.is_neutral() {
			Ok(())
		} else {
			eprintln!("{:?}", out);
			Err("Semaphore is not neutral") // TODO: Make this a bit easier to debug
		}
	})
}

pub fn os_wait(id: SemaphoreId) -> Result<(), ()> {
	// TODO: Syscall, see init_sem
	
	without_interrupts(|| {
		let store = SEMAPHORE_STORE.try_read().expect("DEADLOCK");
		let entry = store.get(&id).ok_or(())?;
		if entry.wait() { // Successfully acquired
			Ok(())
		} else { // Need to wait acquired
			// TODO: There got to be a better solution than locking process manager to get the current pid
			// Maybe have it atomically stored somewhere? What happens in multi-threaded applications?
			// What we need is some way to identify who called us
			// Check what page table / stack frame we are in?
			// Again we have to reference the process manager to know what belongs to who
			// !!!!
			let pm = PROCESS_MANAGER.try_lock()
				.expect("I just knew there was going to be a deadlock here");
			
			if pm.get_current_scheduling_level() == SchedulingLevel::Periodic {
				// Only matters for periodic devices, as they are the only ones that can yield and then preempt
				entry.add_to_wait_queue(pm.get_current_process_name());
			}
			
			drop(pm);
			drop(store);
			
			while {
				enable_int();
				os_yield();
				disable_int();
				let store =
					SEMAPHORE_STORE.try_read().expect("DEADLOCK");
				
				// Since we can't delete semaphores, I just expect this to work
				// Break the loop when we get the lock
				!store.get(&id).unwrap().wait()
			} {}
			
			Ok(())
		}
	})
}

pub fn os_signal(id: SemaphoreId) -> Result<(), ()> {
	without_interrupts(|| {
		let mut pm = PROCESS_MANAGER.try_lock().unwrap();
		let store = SEMAPHORE_STORE.read();
		let sem = store.get(&id).ok_or(())?;
		sem.signal();
		if sem.check_and_pop_if_exists(pm.get_scheduled_name()) {
			pm.unyield_current_periodic();
			drop(store);
			drop(pm);
			enable_int();
			os_yield();
			disable_int();
		}
		
		Ok(())
	})
}

pub fn os_abort() {
	println!("!! OS TERMINATED !!");
	interrupts::disable();
	loop {
		x86_64::instructions::hlt();
	}
}

// #[cfg(test)]
// mod process_test {
// 	use super::*;
// 	#[test]
// 	fn application_tests() {
// 		let fifo_key = os_init_fifo();
// 		// os_create(123, SchedulingLevel::Periodic, 4, write_test_app);
// 		os_create(fifo_key as i32, SchedulingLevel::Periodic, 4, test_app).unwrap();
// 		os_create(fifo_key as i32, SchedulingLevel::Periodic, 3, test_app).unwrap();
// 		os_create(fifo_key as i32, SchedulingLevel::Periodic, 2, test_app).unwrap();
// 		os_create(fifo_key as i32, SchedulingLevel::Periodic, 1, test_app).unwrap();
// 		// os_create(9990, SchedulingLevel::Sporadic, 1,test_app_spor).unwrap();
// 		// os_create(9991, SchedulingLevel::Sporadic, 2,test_app_spor).unwrap();
// 		// os_create(9992, SchedulingLevel::Sporadic, 3,test_app_spor).unwrap();
// 		// os_create(9993, SchedulingLevel::Sporadic, 4,test_app_spor).unwrap();
// 		// os_create(10, SchedulingLevel::Device, 10, test_app_device).unwrap();
// 		// os_create(15, SchedulingLevel::Device, 15, test_app_device).unwrap();
// 	}
// }
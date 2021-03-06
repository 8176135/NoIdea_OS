#![no_main]
#![no_std]

#![feature(alloc_prelude)]
#![feature(option_expect_none)]
#![feature(type_alias_impl_trait)]
#![feature(core_intrinsics)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(llvm_asm)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod vga_buffer;
mod memory;
mod sync;
mod tests;
mod serial;
mod interrupts;
mod gdt;
mod helper;
mod ipc;

// Logic
mod kernel;
mod processes;

// Collections
mod special_collections;

use x86_64::VirtAddr;
use spin::Mutex;
use crate::memory::allocator::BootInfoFrameAllocator;

static FRAME_ALLOCATOR: Mutex<BootInfoFrameAllocator> = Mutex::new(BootInfoFrameAllocator::new());
static TEMP_MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
	
	let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
	let mut mapper = unsafe { memory::paging::init(phys_mem_offset) };
	unsafe { FRAME_ALLOCATOR.lock().init(&boot_info.memory_map, phys_mem_offset) };

	memory::allocator::init_heap(&mut mapper, &mut *FRAME_ALLOCATOR.lock())
		.expect("Failed to init heap");
	*TEMP_MAPPER.lock() = Some(mapper);
	
	kernel::os_init();
	
	#[cfg(test)]
		crate::test_main();
	
	kernel::os_start();
	println!("Didn't quite crash");
	x86_64::instructions::interrupts::enable();
	processes::idle_process();
}

use core::panic::PanicInfo;
use x86_64::structures::paging::OffsetPageTable;

/// This function is called on panic.

// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	x86_64::instructions::interrupts::disable();
	eprintln!("{}", info);

	loop {
		x86_64::instructions::hlt();
	}
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	serial_println!("[failed]\n");
	serial_println!("Error: {}\n", info);
	tests::exit_qemu(tests::QemuExitCode::Failed);
	loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
	panic!("allocation error: {:?}, probably not enough bytes", layout)
}


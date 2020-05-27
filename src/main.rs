#![no_main]
#![no_std]

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
mod kernel;
mod static_collections;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
	
	let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
	let mut mapper = unsafe { memory::paging::init(phys_mem_offset) };
	let mut frame_allocator = unsafe {
		BootInfoFrameAllocator::init(&boot_info.memory_map)
	};
	
	memory::allocator::init_heap(&mut mapper, &mut frame_allocator)
		.expect("Failed to init heap");
	
	
	// allocate a number on the heap
	let heap_value = Box::new(41);
	println!("heap_value at {:p}", heap_value);
	
	// create a dynamically sized vector
	let mut vec = Vec::new();
	for i in 0..500 {
		vec.push(i);
	}
	println!("vec at {:p}", vec.as_slice());
	
	// create a reference counted vector -> will be freed when count reaches 0
	let reference_counted = Rc::new(vec![1, 2, 3]);
	let cloned_reference = reference_counted.clone();
	println!("current reference count is {}", Rc::strong_count(&cloned_reference));
	core::mem::drop(reference_counted);
	println!("reference count is {} now", Rc::strong_count(&cloned_reference));
	
	kernel::os_init();
	
	#[cfg(test)]
		crate::test_main();
	
	let x = Box::new(4);
	
	kernel::os_start();
	
	println!("Didn't quite crash");
	loop {
		x86_64::instructions::hlt();
	}
}

use core::panic::PanicInfo;
use x86_64::VirtAddr;
use x86_64::structures::paging::{PageTable, MapperAllSizes, Mapper, Page};
use crate::memory::paging::BootInfoFrameAllocator;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

/// This function is called on panic.

// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
	panic!("allocation error: {:?}", layout)
}


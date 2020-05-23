#![no_main]
#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
pub extern "C" fn _start() -> ! {
	kernel::os_init();
	
	#[cfg(test)]
		crate::test_main();
	
	kernel::os_start();
	
	println!("Didn't quite crash");
	loop {
		x86_64::instructions::hlt();
	}
}

use core::panic::PanicInfo;
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

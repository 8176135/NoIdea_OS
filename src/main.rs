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

#[no_mangle]
pub extern "C" fn _start() -> ! {
	kernel::os_init();
	
	#[cfg(test)]
		crate::test_main();
	
	fn stack_overflow() {
		stack_overflow(); // for each recursion, the return address is pushed
	}
	
	stack_overflow();
	
	panic!("Didn't quite crash");
	loop {}
}

use core::panic::PanicInfo;
/// This function is called on panic.

// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	eprintln!("{}", info);
	loop {}
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

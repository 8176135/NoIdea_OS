use crate::{println, eprintln};
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
	panic!("!!EXCEPTION!!: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_breakpoint_exception() {
	serial_print!("test_breakpoint_exception...");
	x86_64::instructions::interrupts::int3();
	serial_println!("[ok]");
}
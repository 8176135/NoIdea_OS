use crate::{println, eprintln};
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, error_code: u64) -> ! {
	panic!("!!EXCEPTION!!: DOUBLE FAULT\n{:#?}\nErrCode: {}", stack_frame, error_code);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_handler(stack_frame: &mut InterruptStackFrame, error_code: u64) {
	panic!("EXCEPTION: ALIGNMENT ERROR\n{:#?}\nErrCode: {}", stack_frame, error_code);
}

pub extern "x86-interrupt" fn gp_handler(stack_frame: &mut InterruptStackFrame, error_code: u64) {
	panic!("EXCEPTION: GENERAL PROTECTION ERROR\n{:#?}\nErrCode: {}", stack_frame, error_code);
}

pub extern "x86-interrupt" fn stack_segment_handler(stack_frame: &mut InterruptStackFrame, error_code: u64) {
	println!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nErrCode: {}", stack_frame, error_code);
}

pub extern "x86-interrupt" fn debug_handler(stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn divide_handler(stack_frame: &mut InterruptStackFrame) {
	println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
	stack_frame: &mut InterruptStackFrame,
	error_code: PageFaultErrorCode,
) {
	use x86_64::registers::control::Cr2;
	
	println!("EXCEPTION: PAGE FAULT");
	println!("Accessed Address: {:?}", Cr2::read());
	println!("Error Code: {:?}", error_code);
	println!("{:#?}", stack_frame);
	os_abort();
}

#[cfg(test)]
use crate::{serial_print, serial_println};
use crate::kernel::os_abort;

#[test_case]
fn test_breakpoint_exception() {
	serial_print!("test_breakpoint_exception...");
	x86_64::instructions::interrupts::int3();
	serial_println!("[ok]");
}
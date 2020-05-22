use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259_simple::ChainedPics;
use crate::{println, eprintln};
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = // Remap PIC ports via offset
	spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = create_idt();
}

fn create_idt() -> InterruptDescriptorTable {
	let mut idt = InterruptDescriptorTable::new();
	idt.breakpoint.set_handler_fn(breakpoint_handler);
	idt.double_fault.set_handler_fn(double_fault_handler);
	idt
}

pub fn interrupt_init() {
	IDT.load();
	unsafe { PICS.lock().initialize() }
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
	panic!("!!EXCEPTION!!: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
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
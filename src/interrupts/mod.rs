use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use hardware::InterruptIndex;
use lazy_static::lazy_static;
use crate::println;

mod cpu;
pub mod hardware;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = create_idt();
}

pub fn interrupt_init() {
	IDT.load();
	unsafe { hardware::PICS.lock().initialize() }
	x86_64::instructions::interrupts::enable();
}

pub extern "x86-interrupt" fn syscall(_stack_frame: &mut InterruptStackFrame) {
	println!("yay a syscall happened!");
}

fn create_idt() -> InterruptDescriptorTable {
	let mut idt = InterruptDescriptorTable::new();
	idt.breakpoint.set_handler_fn(cpu::breakpoint_handler);
	idt.page_fault.set_handler_fn(cpu::page_fault_handler);
	unsafe {
		idt.double_fault.set_handler_fn(cpu::double_fault_handler)
			.set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
	}
	idt[InterruptIndex::Timer as u8 as usize].set_handler_fn(hardware::timer_interrupt_handler);
	idt[InterruptIndex::Keyboard as u8 as usize].set_handler_fn(hardware::keyboard_interrupt_handler);
	idt[InterruptIndex::Syscall as u8 as usize].set_handler_fn(syscall);
	idt
}
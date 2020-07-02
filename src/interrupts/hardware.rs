use pic8259_simple::ChainedPics;
use x86_64::structures::idt::InterruptStackFrame;
use lazy_static::lazy_static;
use crate::{print, println};
use crate::processes::PROCESS_MANAGER;
use super::helper_macros::*;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = // Remap PIC ports via offset
	spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
	Timer = PIC_1_OFFSET,
	Keyboard, // +1
}
// _stack_frame: &mut InterruptStackFrame

#[naked]
pub extern "x86-interrupt" fn timer_interrupt_handler() {
	// Do I understand what is going on?
	unsafe {
		interrupt_push!();
		llvm_asm!("cld" ::: : "volatile");
		// llvm_asm!("
		// 	sub    $0x10, %rsp
		// 	cld
		// 	lea    0x58(%rsp), %rax
		// 	mov    %rax, 0x8(%rsp)
		// " ::: "rax" : "volatile");
		
		llvm_asm!( "
			mov %rsp, %rdi //; Pass rsp as first argument
			call ${0:c}
			mov %rax, %rsp
			": : "i"(timer_internal as u64) : "memory", "rsp" : "volatile", "alignstack");
		// NOTE: have to cast timer_interval as u64 because of a nightly rust compiler error... I think
		
		
		// let new_stack_pointer = timer_internal(stack_pointer);
		
		// llvm_asm!( "
		// 	mov $0, %rsp
		// 	":: "r"(new_stack_pointer) : "rsp" : "volatile");
		
		// llvm_asm!("
		// 	add    $0x10, %rsp
		// " ::: "rsp" : "volatile");
		interrupt_pop!();
		llvm_asm!("iretq" :::: "volatile");
	}
}

// Has to be C calling convention
#[inline(never)]
unsafe extern "C" fn timer_internal(stack_p: usize) -> usize {
	
	// println!("YAY");
	let new_stack_pointer = PROCESS_MANAGER.try_lock()
		.and_then(|mut p_manager| p_manager.next_tick_preempt_process(stack_p))
		.map(|c| c.as_u64() as usize)
		.unwrap_or(stack_p);
	
	PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
	
	new_stack_pointer
}
// #[inline(never)]
// pub extern "C" fn random_thing(_stack_frame: usize) {
// 	let stack_pointer: usize;
// 	unsafe {
// 		llvm_asm!( "
// 	mov %rsp, $0
// 	": "={rsp}"(stack_pointer): :
// 	 "rax", "rcx", "rdx", "rbx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15" :
// 	 "volatile");
//
// 		let return_pointer = crate::scheduling::check_schedule(stack_pointer);
//
// 		llvm_asm!( "
// 	mov $0, %rsp
// 	": : "{rsp}" (return_pointer):
// 	 "rax", "rcx", "rdx", "rbx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15" :
// 	 "volatile");
// 	}
// }

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
	use x86_64::instructions::port::Port;
	use spin::{Mutex, MutexGuard};
	use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
	
	lazy_static! {
		// US104Key has 1 row high enter key
		static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
			Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
	}
	
	let mut port = Port::new(0x60);
	let scancode: u8 = unsafe { port.read() };
	
	let mut keyboard: MutexGuard<Keyboard<_, _>> = KEYBOARD.lock();
	if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
		if let Some(key) = keyboard.process_keyevent(key_event) {
			match key {
				DecodedKey::Unicode(character) => print!("{}", character),
				DecodedKey::RawKey(key) => print!("{:?}", key),
			}
		}
	}
	
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
	}
}
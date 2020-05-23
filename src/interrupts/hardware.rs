use pic8259_simple::ChainedPics;
use x86_64::structures::idt::InterruptStackFrame;
use lazy_static::lazy_static;
use crate::{print, println};

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

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
	print!(".");
	unsafe {
		PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
	}
}

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
	use x86_64::instructions;
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
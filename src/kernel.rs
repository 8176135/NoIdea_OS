use crate::interrupts::interrupt_init;

pub fn os_init() {
	interrupt_init();
	
}
#[macro_use]
mod helper_macros;

use x86_64::structures::idt::InterruptDescriptorTable;
use hardware::InterruptIndex;
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use core::convert::TryFrom;
use x86_64::VirtAddr;
use crate::println;

mod cpu;
pub mod hardware;

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u64)]
pub enum SyscallCommand {
	Yield = 10,
	Terminate,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = create_idt();
}

pub fn interrupt_init() {
	IDT.load();
	unsafe { hardware::PICS.lock().initialize() }
}

// TODO: Actually have a kernel stack pointer
// The user calling this syscall better have interrupt disabled, how are they going to do that in usermode?
// who knows, good thing everyone is in kernel mode I suppose.
#[naked]
pub unsafe extern fn syscall_handler() -> ! {
	// x86_64::instructions::interrupts::disable();
	// Make sure not to use any registers, somehow
	llvm_asm!("
		  swapgs // Load the TSS as temporary storage lol
		  mov qword ptr gs:[28], rsp // Move rsp to temporary 'reserved' location in the TSS
		  push 0  // I think this should be 0, it works with 0.
		  push qword ptr gs:[28] // Push original rsp
		  mov qword ptr gs:[28], 0 // Clear the reserved section again
          push r11 // Push rflags
          mov r11, cs // Move cs to temporary register to be pushed, we already pushed r11
          push r11 // Push code segment
          push rcx // Push return pointer
          "
          :
          :
          :
          : "intel", "volatile");
	
	interrupt_push!();
	// TODO: Make everything use the same syntax
	
	llvm_asm!("
		mov rdi, rsp // Store process rsp as first argument
		mov rsp, qword ptr gs:[4] // Get the ring 0 stack pointer
		swapgs // Move gs back to TSS
	"
	:
	:
	:
	: "intel", "volatile");
	
	llvm_asm!( "
			mov %rax, %rsi //; Pass rax (first argument of syscall) as second argument
			call ${0:c}
			// I don't think we need to save the kernel stack pointer...
			mov %rax, %rsp // Use return number as stack pointer
			": : "i"(internal_syscall as u64) : "memory", "rsp", "rdi", "rsi", "rax" : "volatile", "alignstack");
	
	interrupt_pop!();
	// TODO: There is a lot of things wrong here, we are assuming everything is just in kernel space.
	llvm_asm!("iretq");
	unreachable!();
}

#[inline(never)]
extern "C" fn internal_syscall(stack_p: usize, call_num: usize) -> usize {
	use crate::processes::PROCESS_MANAGER;
	
	let call_num = SyscallCommand::try_from(call_num as u64)
		.expect("Invalid Syscall Number");
	
	match call_num {
		SyscallCommand::Yield => {
			PROCESS_MANAGER.try_lock().expect("Disabled interrupts here, need to deal with locked PM")
				.yield_current_process(VirtAddr::new(stack_p as u64)).as_u64() as usize
		},
		SyscallCommand::Terminate => {
			PROCESS_MANAGER.try_lock().expect("Disabled interrupts here, need to deal with locked PM")
				.end_current_process().as_u64() as usize
		}
	}
}

#[inline(always)]
pub fn syscall1(call_num: SyscallCommand) -> u64 {
	let ret: u64;
	unsafe {
		llvm_asm!("syscall" : "={rax}" (ret) : "{rax}" (call_num as u64) : "rcx", "r11", "memory" : "volatile");
	}
	ret
}

fn create_idt() -> InterruptDescriptorTable {
	let mut idt = InterruptDescriptorTable::new();
	idt.breakpoint.set_handler_fn(cpu::breakpoint_handler);
	idt.page_fault.set_handler_fn(cpu::page_fault_handler);
	idt.alignment_check.set_handler_fn(cpu::alignment_handler);
	idt.debug.set_handler_fn(cpu::debug_handler);
	idt.divide_error.set_handler_fn(cpu::divide_handler);
	idt.general_protection_fault.set_handler_fn(cpu::gp_handler);
	idt.stack_segment_fault.set_handler_fn(cpu::stack_segment_handler);
	
	unsafe {
		idt.double_fault.set_handler_fn(cpu::double_fault_handler)
			.set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
	}
	
	// Hack to get around compiler check
	// (We have to do this because we are removing an argument, which we weren't using)
	// But rust debug builds have a "bug"? where naked functions are not actually naked
	let timer_interrupt_address = hardware::timer_interrupt_handler as *const ();
	idt[InterruptIndex::Timer as u8 as usize].set_handler_fn(unsafe { core::mem::transmute(timer_interrupt_address) });
	idt[InterruptIndex::Keyboard as u8 as usize].set_handler_fn(hardware::keyboard_interrupt_handler);
	idt
}
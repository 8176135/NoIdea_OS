use crate::kernel::Pid;
use crate::println;
use crate::registers::Registers;
use crate::kernel::Process;

// pub fn save_state(old_process: &mut Process) {
//
// }
//
// pub fn context_switch(new_process: &mut Process, old_pid: Option<&mut Process>, old_registers: Option<Registers>) {
// 	if let Some(old_pid) = old_pid {
// 		let rax: u64 = 0;
// 		let rcx: u64 = 0;
// 		let rdx: u64 = 0;
// 		let rbx: u64 = 0;
// 		let rsp: u64 = 0;
// 		let rbp: u64 = 0;
// 		let rsi: u64 = 0;
// 		let rdi: u64 = 0;
//
// 		unsafe {
// 			llvm_asm!("mov %rdi, $0" : "=r" (rdi));
// 			llvm_asm!("mov %rax, $0" : "=r" (rax));
// 			llvm_asm!("mov %rdx, $0" : "=r" (rdx));
// 			llvm_asm!("mov %rcx, $0" : "=r" (rcx));
// 			llvm_asm!("mov %rsi, $0" : "=r" (rsi));
// 			llvm_asm!("mov %rsp, $0" : "=r" (rsp));
// 			llvm_asm!("mov %rbx, $0" : "=r" (rbx));
// 			llvm_asm!("mov %rbp, $0" : "=r" (rbp));
// 		}
//
// 		let regs = Registers {
// 			rax,
// 			rcx,
// 			rdx,
// 			rbx,
// 			rsp,
// 			rbp,
// 			rsi,
// 			rdi
// 		};
//
// 		println!("DOES THIS WORK: {:?}", regs);
// 		old_pid.set_regs(regs);
// 	}
// }
use crate::println;

#[inline(always)]
pub fn print_stack_pointer() {
	let other: usize;
	unsafe {
		llvm_asm!( "
	mov %rsp, $0
	": "={rsp}"(other): :
	 : "volatile");
	}
	println!("{:x}", other);
}

.intel_syntax noprefix

.global asm_fake_register

//; asm_fake_register(new_stack_pointer: usize, terminate_func_addr: usize, program_start_addr: usize) -> usize
asm_fake_register:
	mov r8, rsp // Store stack pointer in a unused scratch register
	mov rsp, rdi // Set our stack pointer to the new stack pointer passed in the first argument

	push rsi // Push os_terminate address
	mov rcx, rsp // Update rcx to contain the stack pointer before our fake "interrupt"
	mov rax, 0 //; Apparently you should use the XOR method
	push rax // Push ss (it's just zero)
	push rcx // Push the stack pointer just before our fake "interrupt"
    pushfq   //; push RFLAGS register to stack TODO: (Currently this is "fine" (it really isn't, since all the other flags will be wrong) since everything is in ring 0, but we need to customize this once we have rings)
	mov rcx, cs // Move cs to temporary register to be pushed
	push rcx // Push code segment
	push rdx // Push instruction pointer

	// Use the more efficient XOR method to zero out registers

	// Push a bunch of zeros for all the registers
	push rdi //; rbp  // Push the input stack start address to the base stack pointer
	push rax //; r15
	push rax //; r14
	push rax //; r13
	push rax //; r12
	push rax //; r11
	push rax //; r10
	push rax //; r9
	push rax //; r8
	push rax //; rdi
	push rax //; rsi
	push rax //; rdx
	push rax //; rcx
	push rax //; rbx
	push rax //; rax

	sub rsp, <RSP_SUB_VAL> // Change based on build mode in build.rs (This is so stupid, we need a better way of doing this)
	mov rax, rsp // Save new stack pointer in rax, the C return register
	mov rsp, r8 // Get back our original stack pointer

    ret
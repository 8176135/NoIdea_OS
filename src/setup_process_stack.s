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
    pop rcx // Move RFLAGs to a temporary register for modification
    or rcx, 0x0200 // Enable interrupts in the fake rflags
    push rcx // push it back onto the stack
	mov rcx, cs // Move cs to temporary register to be pushed
	push rcx // Push code segment
	push rdx // Push instruction pointer

	// TODO: move this out of assembly file into naked function
	// Push a bunch of zeros for all the registers 15
	push rdi //; rbp  // Push the input stack start address to the base stack pointer
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;
	push rax //;

	// NO, just use naked function like a normal person
	//sub rsp, <RSP_SUB_VAL> // Change based on build mode in build.rs (This is so stupid, we need a better way of doing this)
	mov rax, rsp // Save new stack pointer in rax, the C return register
	mov rsp, r8 // Get back our original stack pointer

    ret
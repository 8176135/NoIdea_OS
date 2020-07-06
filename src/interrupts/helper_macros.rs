// Some helper macros taken from RedoxOS

macro_rules! preserved_push {
    () => (llvm_asm!(
        "push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15"
        : : : : "intel", "volatile"
    ));
}

macro_rules! preserved_pop {
    () => (llvm_asm!(
        "pop r15
        pop r14
        pop r13
        pop r12
        pop rbp
        pop rbx"
        : : : : "intel", "volatile"
    ));
}
// // Load kernel tls
// mov rax, 0x18
// mov fs, ax // can't load value directly into `fs`
// macro_rules! fs_push {
//     () => (llvm_asm!(
//         "
//         push fs
//         "
//         : : : : "intel", "volatile"
//     ));
// }
//
// macro_rules! fs_pop {
//     () => (llvm_asm!(
//         "pop fs"
//         : : : : "intel", "volatile"
//     ));
// }

macro_rules! scratch_push {
    () => (llvm_asm!(
        "push rax
        push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11"
        : : : : "intel", "volatile"
    ));
}

macro_rules! scratch_pop {
    () => (llvm_asm!(
        "pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi
        pop rdx
        pop rcx
        pop rax"
        : : : : "intel", "volatile"
    ));
}

macro_rules! interrupt_push {
    () => {
        scratch_push!();
        preserved_push!();
        // fs_push!();
    };
}
macro_rules! interrupt_pop {
    () => {
        // fs_pop!();
        preserved_pop!();
        scratch_pop!();
    };
}
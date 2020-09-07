## Design notes

If a sporadic process yields, but there are no other processes available. We just return control directly back to that process.

---

### Quick descriptions of the dependencies

- `bootloader`: Connects to the rust boot image, this is just to get the type for the struct passed by the bootloader.
- `volatile`: Thin wrapper around the volatile instructions to avoid compiler optimisations.
- `x86_64`: A lot of structures for working with x86 to prevent the need for using random magic number. Also a bunch of paging logic.
- `uart_16550`: Structures for the uart_16550 serial output, used to help with debugging.
- `pic8259_simple`: Structures for the CPU's programmable interface controller.
- `pc-keyboard`: Structures and translation for the keyboard events.
- `lazy_static`: General Rust helper library to have safe, lazily runtime initialized, static variables.
- `num_enum`: General Rust helper library, allows conversion from numbers to enums.
- `spin`: Spinlock mutex. Interrupt semaphores doesn't work for a lot of synchronization stuff.

### Build/Run Instructions:
#### Requirements: 
- Rust Nightly (Easiest way is to get [rustup](https://rustup.rs/), then run `rustup install nightly`)
- Cargo-xbuild (`cargo install cargo-xbuild` After installing rust)
- Bootimage runner (`cargo install bootimage`)
- Qemu
- Maybe more? Look at the error messages I suppose

#### Steps:
- Run `cargo xrun --release`.
- Unpause the qemu instance

## OS Design

> Note: The top of main.rs contains 
>```
>#![no_main]
>#![no_std]
>```
>This indicates that no Rust standard library is imported, and no C main function is defined a for the linker.
>
>This is then followed by a ton of nightly unstable features. See [TODO] for what each is required for.

### Boot process

The bootloader calls the `_start` function in `main.rs` after the entire kernel is loaded into memory. 
The bootloader does physical to virtual mapping before `_start` is called. 
Some parts of the memory like `0xb8000` have identity mapping, while kernel code and stack are mapped arbitrarily.
The bootloader also comes with some options to configure how to map the rest of the memory, I
configured it to also map the entire physical memory into virtual memory at a very high offset. The exact offset and other memory information
is passed in via the `&'static bootloader::BootInfo` argument in `_start`. This means that if the offset is `0x100000000`, 
then both `0xb8000` and `0x1000b8000` are mapped to the exact same position in memory. 
This does break some of the rust memory safety guarantees if not careful.

This full physical memory mapping is used mainly to get access to page table. 
The page table physical address is stored in the `Cr3` Register, which means to view and modify the page table, we make 
use of the full physical mapping done by the bootloader. 

Then the frame allocator and the global heap allocator is then initialized, `HEAP_SIZE` const contains the size of allowed heap memory of the entire OS 
for the allocator. This must be lower than the available memory on the system, as the frames for the heap is allocated at OS boot up.
Defining and initializing a global allocator allows us to bring back part of the standard library as the `alloc` library.

We then setup and load the GDT (Global Descriptor Table), TSS (Task State Segment), and IDT (Interrupt Descriptor Table).
Here we define where our stack is going to be for interrupts, what hardware and software interrupts to catch...etc.
Of specific note the timer interrupt is defined here, and it is what will govern context switches.

Next we write to the x86_64 model specific registers `LStar, SFMask, KernelGsBase, Star, Efer`, so that we can handle syscalls. 
(Explained later in the syscall section)

Finally, if we are running this as a test, it will run the kernel tests (doesn't really work at the moment).
otherwise, we run `os_start()`, which will schedule the startup processes and return. After that, we enable interrupts, 
then run the idle process which is just a infinite loop with `hlt` to save CPU processing power. 
We wait here until the timer interrupts begins the scheduling of the processes.

### Memory Management
Currently there are 2 allocators for the OS. One is the `BootInfoFrameAllocator`, which allocates fixed size physical frames
of 4KiB, for use by pages to map memory to. Then there is the `FixedSizeBlockAllocator`, which, counter to what the name suggests,
is actually for allocating dynamically sized pieces of memory for the kernel heap. This allocator is also currently allocating heap memory
for each program as well, as everything is compiled together. Once the programs are actually separate pieces of code, they will probably
have their own allocators. 

`BootInfoFrameAllocator`: Uses a free list for a fast linear allocation time proportional to the number of frames to be allocated. 
If there are no more frames in the free list, it look for usable regions in the memory map passed by the boot loader.
 
`FixedSizeBlockAllocator`: Block based allocator that will allocate continuous block/chunks of virtual memory in powers of 2. 
This will split up bigger blocks of memory into smaller pieces to more efficiently fill the requirements. Currently does not join smaller blocks
back together into bigger blocks, so eventually you will run out of big chunks of continuous memory.
 
Further improvements can be made here to join together smaller blocks to make larger blocks when needed. 
Apparently this is called a "Buddy Allocator". 
Another thing is that the current implementation the largest continuous allocation is the size of the largest block. A 
solution to this would be to use paging for bigger allocations.

Stack memory is allocated for each process with a different address. Currently every process share the same memory, 
with just different offsets for where their stack begins from, so any process can access the memory of any other process.
Improvements would be to have the same virtual stack location for every process, but swap out pages so it points to different physical memory. 

### Interrupts

The `create_idt()` function creates a Interrupt Descriptor Table. We use the structure provided by `x86_64` crate, 
to prevent us messing up the bit shifts to create the IDT entry, and also to verify the signature of functions passed as handlers.

Here we register a bunch of CPU exception handlers like divide by zero, page fault...etc. 
Of special note, we also handle double faults, which is called when a fault happens inside a exception handler for the first fault,
or if the first fault/exception does not have a handler. The double fault handler also needs a custom stack pointer, because if 
something a page fault happens with a invalid stack position, it will cause a second fault in the page fault handler.

We also register some hardware/PIC registers, specifically the keyboard and the timer.

The syscall handler behaviour is registered in the `LStar, SFMask, KernelGsBase, Star, Efer, EferFlags` model_specific specific registers.

Most of the interrupt handlers use the `x86-interrupt` calling convention, where the compiler will save all the registers 
used in the function body, and restore them at the end, along with the necessary interrupt flags.
This doesn't work for handlers that need to do process switches, which is explained later.

### Process Management

Process management data is stored in the one static `PROCESS_MANAGER` variable, which is controlled by a spinlock Mutex, 
to make sure of synchronisation.

#### Process Creation:

1. Get free PID from incrementing pool
2. Allocate some stack space
3. Swap to the new process stack.
4. Push the terminate function address, so when returning from the application will call the terminate process syscall
5. Modify the stack to fake that an interrupt has happened.
6. Push all the registers, currently just pushing 0 to everything
7. Return the modified stack pointer to put in the PCB.
8. Store the PCB in PROCESS_MANAGER, and the pid in the relevant scheduling structures.

### Task Switching

The overall idea for task switching is instead of saving all the registers in the PCB, we push them onto the stack, and 
just save the stack pointer. When switching, all we need to do is change the stack pointer, and pop all the stack registers.
When creating a new process, we need to fake the stack so when we switch the stack pointer to a newly created process, 
the same logic as restoring a interrupted process can be applied.

### Syscalls

Every syscall command on the system is stored in the enum in `interrupts::SyscallCommand`. Currently there are only 3 commands.
- Terminate (self)
- Yield (self)
- TerminateEverythingElse (only implemented because we don't have a good way to list all processes, or terminate other processes)

The syscall handler is marked as a `#[naked]` function, meaning that there is no function prologue and epilogue is generated.
This allows us to manage exactly what registers to push, what order to push them, and everything else.

The same logic for task switching is used in the syscall as well.

### Semaphores
The semaphores mostly follow the `kernel.h` definitions, with the exception of processes being able to call the semaphore
even if they technically don't own it. This way allows us to easily signal that a process has finished work.

I make the assumption that no *Device* scheduled process is allowed to use a semaphore that can block, since if this is the case,
it will no longer have a "very short execution time". 

When a process calls `os_wait()`, and the semaphore counter smaller or equal to 0, it will be put into a wait queue, 
and call `os_yield()`. However, it turns out because of the behaviour of os_yield(), only periodic processes need to be
on the queue. When a semaphore is freed, we only need to check if the current periodic timeslot process is in the wait queue. 
If not, we can continue execution of the process that called `os_signal()`, as yielding won't allow any process
with higher priority to continue.

To clarify, think of the following scenarios:

| Process calling `os_wait()` | Process calling `os_signal()` | Result                                                                                                                                                         |
|-----------------------------|-------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Periodic                    | Periodic                      | No need to yield, one periodic process can never manually yield to another periodic process, since they always occupy different time slots.                    |
| Sporadic                    | Sporadic                      | No need to yield, every sporadic process has the same priority. Unless it manually yields, there is no need to force it to yield to another sporadic process.  |
| Sporadic                    | Periodic                      | No need to yield, periodic processes has higher priority than sporadic.                                                                                        |
| Periodic                    | Sporadic                      | Only need to yield if the current timeslot contains a periodic process that is blocked on this specific semaphore.                                             |

And *Device* processes never has semaphores as explained above.

### FIFO IPC
The First-In-First-Out Inter-Process-Communication is just a btree with dequeues to be filled with data, and some synchronization code.

## Further work
- Make heap allocator per process
- Add actual swapping in and out of pages for each process.
- Unify stack and heap virtual address, since pages are the ones being swapped (or maybe not to prevent memory attacks).
- Allow bigger continuous allocation sizes.
- Dynamically allocate heap as is needed.
- Floating point doesn't work yet. 
It shouldn't be too hard to make work, since we just need to push
the floating point registers to the stack when task switching.
---

## Tests
All test applications are located in the `tests` folder/module.
`tests/applications.rs` contains the actual applications to run, and
tests runner is in `tests/app_test_runner.rs`. Because of the nature
of scheduling, it is really hard to actually put asserts in to test. 
So currently the best you can do is eyeball the output and see if everything looks right.
The apps also will test memory, IPC, semaphores, 
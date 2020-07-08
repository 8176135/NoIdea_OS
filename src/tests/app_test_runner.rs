use super::applications::*;
use crate::kernel::*;
use crate::processes::SchedulingLevel;
use crate::sync::SemaphoreId;
use crate::println;
use crate::ipc::FifoKey;
use crate::interrupts::{syscall1, SyscallCommand};

pub const TEST_SEMAPHORE_ID: SemaphoreId = 1024;
// const PRINT_FIFO_KEY: FifoKey = 2048;

pub extern "C" fn run_tests() {
	println!("Initiating tests");
	let fifo_key = os_init_fifo();
	// Test case complete signaler
	os_init_sem(TEST_SEMAPHORE_ID, required_count(2));
	println!("Signal test ...");
	os_create(123, SchedulingLevel::Sporadic, 1, test_app_signals).unwrap();
	wait_and_reset_semaphore(TEST_SEMAPHORE_ID, 2);
	println!("Signal test complete");
	println!("IPC test ...");
	os_create(123, SchedulingLevel::Sporadic, 4, write_test_app).unwrap();
	wait_and_reset_semaphore(TEST_SEMAPHORE_ID, 10);
	println!("IPC test Complete");
	println!("Scheduling test ...");
	os_create(fifo_key as i32, SchedulingLevel::Periodic, 4, test_app).unwrap();
	os_create(fifo_key as i32, SchedulingLevel::Periodic, 3, test_app).unwrap();
	os_create(fifo_key as i32, SchedulingLevel::Periodic, 2, test_app).unwrap();
	os_create(fifo_key as i32, SchedulingLevel::Periodic, 1, test_app).unwrap();
	os_create(9990, SchedulingLevel::Sporadic, 1,test_app_spor).unwrap();
	os_create(9991, SchedulingLevel::Sporadic, 2,test_app_spor).unwrap();
	os_create(9992, SchedulingLevel::Sporadic, 3,test_app_spor).unwrap();
	os_create(9993, SchedulingLevel::Sporadic, 4,test_app_spor).unwrap();
	os_create(10, SchedulingLevel::Device, 10, test_app_device).unwrap();
	os_create(15, SchedulingLevel::Device, 15, test_app_device).unwrap();
	os_wait(TEST_SEMAPHORE_ID);
	println!("All tests complete, time to kill everything");
	syscall1(SyscallCommand::TerminateEverythingElse);
	println!("Ah finally some quiet, try typing some stuff");
	
}

fn required_count(i: i32) -> i32 {
	assert!(i > 0);
	(i * -1) + 1
}

fn wait_and_reset_semaphore(id: SemaphoreId, new_num: i32) {
	os_wait(id);
	os_signal(id);
	os_drop_sem(id).expect("Failed to drop semaphore");
	os_init_sem(id, required_count(new_num));
}
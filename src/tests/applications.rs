use crate::kernel::*;
use alloc::prelude::v1::{Vec, ToOwned, String, ToString};
use crate::processes::SchedulingLevel;
use super::app_test_runner::TEST_SEMAPHORE_ID;
use crate::println;

pub extern "C" fn test_app() {
	use alloc::format;
	
	let mut a: i64 = 0;
	let arg = os_getparam();
	let mut stuff: Vec<u8> = Vec::new();
	for i in 0..5000000 {
		a = a.wrapping_add(i);
	}
	
	// println!("Enter the gates: {} {}", os_getparam(), a);
	os_write(arg as u32, format!("Periodic Test: {}", a).as_bytes()).unwrap();
	stuff.resize(20000, 0);
	os_write(arg as u32, "Ends".as_bytes()).unwrap();
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn test_app_printer() {
	use alloc::format;
	
	let mut a: i64 = 0;
	let arg = os_getparam();
	let mut stuff: Vec<u8> = Vec::new();
	for i in 0..5000000 {
		a = a.wrapping_add(i);
	}
	
	stuff.resize(20000, 0);
	let read_size = os_read(arg as u32, &mut stuff).unwrap();
	stuff.truncate(read_size);
	let read_data = String::from_utf8_lossy(&stuff);
	println!("Test App String Read: {}", read_data);
	// println!("Enter the gates: {} {}", os_getparam(), a);/
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn test_app_signals() {
	println!("Before Init");
	os_init_sem(123, 1).unwrap();
	os_wait(123).unwrap();
	os_create(123, SchedulingLevel::Sporadic, 4, test_app_signals_recv).unwrap();
	println!("Start Work");
	os_yield();
	do_work();
	println!("Then A");
	os_signal(123);
	os_yield();
	do_work();
	println!("Trying for semaphore again");
	os_wait(123);
	println!("Fin");
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn test_app_signals_recv() {
	println!("Before!!");
	os_wait(123).unwrap();
	println!("After");
	do_work();
	do_work();
	println!("After Work");
	os_signal(123);
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn write_test_app() {
	let stuff = "Yep this works".to_owned();
	
	let fifo_stuff = os_init_fifo();
	os_create(fifo_stuff as i32, SchedulingLevel::Sporadic, 10, read_test_app).unwrap();
	
	os_write(fifo_stuff, &postcard::to_allocvec(&stuff).unwrap()).unwrap();
	os_write(fifo_stuff, &postcard::to_allocvec(&"STUFF".to_owned()).unwrap()).unwrap();
	os_write(fifo_stuff, &postcard::to_allocvec(&stuff).unwrap()).unwrap();
	println!("Write!! {}", os_getparam());
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn read_test_app() {
	let fifo_stuff = os_getparam() as u32;
	let mut stuff_output = [0u8; 40];
	let length_read = os_read(fifo_stuff, &mut stuff_output).unwrap();
	let (out, remaining): (String, _) =
		postcard::take_from_bytes(&stuff_output).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	let (out, remaining): (String, _) =
		postcard::take_from_bytes(remaining).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	let (out, _remaining): (String, _) =
		postcard::take_from_bytes(remaining).expect("Failed to deserialize msg");
	println!("Read!! {:?}", out);
	println!("Read!! {}", length_read);
	os_signal(TEST_SEMAPHORE_ID);
}

pub extern "C" fn test_app_spor() {
	let mut a: i64 = 0;
	let param = os_getparam();
	for i in 0..10_000_000 {
		// if i % 100000 == 0 {
		// 	println!("{}", param);
		// }
		if i % 1_000_000 == 0 {
			println!("Hey, progress {} {}", param, i / 1_000_000);
			os_yield();
		}
		a = a.wrapping_add(i);
	}
	os_create(param + 4, SchedulingLevel::Sporadic, 123, test_app_spor).unwrap();
	println!("WE OUT!! {}", param);
	if param <= 9993 {
		os_signal(TEST_SEMAPHORE_ID);
	}
}

pub extern "C" fn test_app_device() {
	let param = os_getparam();
	for i in 0..20 {
		println!("Reminder for stuff from {} {} ", param, i);
		os_yield();
	}
	os_create(param + 10, SchedulingLevel::Device, (param + 10) as u64, test_app_device).unwrap();
	if param == 20 || param == 25 {
		os_signal(TEST_SEMAPHORE_ID);
	}
}

pub extern "C" fn big_memory() {
	let param = os_getparam();
	let mut array_of_arrays = alloc::vec::Vec::with_capacity(16);
	println!("about to allocate: {} + some more bytes", param);
	for x in (0..param).step_by(4096 * 4) {
		let mut array = alloc::vec::Vec::with_capacity((4096 * 4).min(param - x) as usize);
		array.resize((4096 * 4).min(param) as usize, 255u8);
		array_of_arrays.push(array);
	}
	os_signal(TEST_SEMAPHORE_ID);
}

fn do_work() {
	let mut a = volatile::Volatile::new(0i32);
	for i in 0..5000000 {
		if i % 2000000 == 0 {
			println!("{}", i);
		}
		a.write(a.read().wrapping_add(i));
	}
}

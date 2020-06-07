pub mod fixed_pow2_block;

use x86_64::structures::paging::{Mapper, Size4KiB, FrameAllocator, mapper::MapToError, Page, PageTableFlags};
use x86_64::VirtAddr;

//  These are virtual addresses
pub const HEAP_START: usize = 0x_1313_1313_0000;
pub const HEAP_SIZE: usize = 64 * 1024; // 64 KiB

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>)
	-> Result<(), MapToError<Size4KiB>> {
	// Pages contains virtual address we want to map
	let page_range = {
		let heap_start = VirtAddr::new(HEAP_START as u64);
		let heap_end = heap_start + HEAP_SIZE - 1u64;
		let heap_start_page = Page::containing_address(heap_start);
		let heap_end_page = Page::containing_address(heap_end);
		
		Page::range_inclusive(heap_start_page, heap_end_page)
	};
	
	for page in page_range {
		// Allocating a physical address for the said virtual addresses in pages to map to
		let frame = frame_allocator
			.allocate_frame()
			.ok_or(MapToError::FrameAllocationFailed)?;
		let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
		unsafe {
			// Pass the allocator so that page tables can also be mapped if need be
			mapper.map_to(page, frame, flags, frame_allocator)?.flush()
		};
	}
	unsafe {
		ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
	}
	Ok(())
}

use fixed_pow2_block::FixedSizeBlockAllocator;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(
	FixedSizeBlockAllocator::new());

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
	(addr + align - 1) & !(align - 1)
}

pub struct Locked<A> {
	inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
	pub const fn new(inner: A) -> Self {
		Locked {
			inner: spin::Mutex::new(inner),
		}
	}
	
	pub fn lock(&self) -> spin::MutexGuard<A> {
		self.inner.lock()
	}
}

#[cfg(test)]
mod test {
	use crate::{serial_println, serial_print};
	use super::HEAP_SIZE;
	use alloc::boxed::Box;
	
	#[test_case]
	fn simple_allocation() {
		serial_print!("simple_allocation... ");
		let heap_value_1 = Box::new(41);
		let heap_value_2 = Box::new(13);
		assert_eq!(*heap_value_1, 41);
		assert_eq!(*heap_value_2, 13);
		serial_println!("[ok]");
	}
	
	use alloc::vec::Vec;
	
	#[test_case]
	fn large_vec() {
		serial_print!("large_vec... ");
		let n = 1000;
		let mut vec = Vec::new();
		for i in 0..n {
			vec.push(i);
		}
		assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
		serial_println!("[ok]");
	}
	
	#[test_case]
	fn many_boxes() {
		serial_print!("many_boxes... ");
		for i in 0..HEAP_SIZE {
			let x = Box::new(i);
			assert_eq!(*x, i);
		}
		serial_println!("[ok]");
	}
	
	#[test_case]
	fn many_boxes_long_lived() {
		serial_print!("many_boxes_long_lived... ");
		let long_lived = Box::new(1);
		for i in 0..HEAP_SIZE {
			let x = Box::new(i);
			assert_eq!(*x, i);
		}
		assert_eq!(*long_lived, 1);
		serial_println!("[ok]");
	}
}
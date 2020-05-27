use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::structures::paging::{Mapper, Size4KiB, FrameAllocator, mapper::MapToError, Page, PageTableFlags};
use x86_64::VirtAddr;
use linked_list_allocator::LockedHeap;

//  These are virtual addresses
pub const HEAP_START: usize = 0x_1313_1313_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

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

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
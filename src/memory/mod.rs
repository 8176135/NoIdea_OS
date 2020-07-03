use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, Size4KiB, mapper, FrameAllocator, Mapper, FrameDeallocator};
use x86_64::structures::paging::page::PageRange;
use crate::println;

pub mod paging;
pub mod allocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackBounds {
	start: VirtAddr,
	end: VirtAddr,
}

impl StackBounds {
	
	pub const fn zero() -> StackBounds {
		StackBounds {
			start: VirtAddr::zero(),
			end: VirtAddr::zero()
		}
	}
	
	pub fn page_range(&self) -> PageRange<Size4KiB> {
		Page::range(Page::from_start_address(self.start).unwrap(),
					Page::from_start_address(self.end).unwrap())
	}
	
	pub fn start(&self) -> VirtAddr {
		self.start
	}
	
	pub fn end(&self) -> VirtAddr {
		self.end
	}
}

/// We don't have a way to come back down, real question is do we need to? This is all virtual address
fn reserve_stack_memory(size_in_pages: u64) -> Page {
	use core::sync::atomic::{AtomicU64, Ordering};
	static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_5555_5555_0000);
	let start_addr = VirtAddr::new(STACK_ALLOC_NEXT.fetch_add(
		size_in_pages * Page::<Size4KiB>::SIZE,
		Ordering::Relaxed,
	));
	Page::from_start_address(start_addr)
		.expect("`STACK_ALLOC_NEXT` not page aligned")
}

pub	fn dealloc_stack(
	stack_bounds: StackBounds,
	mapper: &mut impl Mapper<Size4KiB>,
	frame_deallocator: &mut impl FrameDeallocator<Size4KiB>) {
	
	for p in stack_bounds.page_range() {
		let a =  x86_64::instructions::interrupts::are_enabled();
		// FIXME: WHY ARE INTERRUPTS ENABLED HERE
		let (frame, flush) =
			mapper.unmap(p).expect("Failed to unmap page");
		flush.flush(); // Is this needed?
		unsafe {
			frame_deallocator.deallocate_frame(frame); // TODO: does nothing atm
		}
	}
}

pub fn alloc_stack(
	size_in_pages: u64,
	mapper: &mut impl Mapper<Size4KiB>,
	frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<StackBounds, mapper::MapToError<Size4KiB>> {
	use x86_64::structures::paging::PageTableFlags as Flags;
	
	let guard_page = reserve_stack_memory(size_in_pages + 1);
	let stack_start = guard_page + 1;
	let stack_end = stack_start + size_in_pages;
	
	for page in Page::range(stack_start, stack_end) {
		let frame = frame_allocator
			.allocate_frame()
			.ok_or(mapper::MapToError::FrameAllocationFailed)?;
		let flags = Flags::PRESENT | Flags::WRITABLE;
		unsafe {
			mapper.map_to(page, frame, flags, frame_allocator)?.flush();
		}
	}
	
	Ok(StackBounds {
		start: stack_start.start_address(),
		end: stack_end.start_address(),
	})
}
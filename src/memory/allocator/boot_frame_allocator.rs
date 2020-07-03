use x86_64::structures::paging::{PhysFrame, Size4KiB, FrameDeallocator, FrameAllocator};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{VirtAddr, PhysAddr};

type PhyFrameIterator = impl Iterator<Item=PhysFrame>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
	memory_map: Option<&'static MemoryMap>,
	phys_frame_iter: Option<PhyFrameIterator>,
	free_list_loc: u64,
	free_list_length: usize,
	phys_memory_offset: VirtAddr,
}

impl BootInfoFrameAllocator {
	pub const fn new() -> Self {
		BootInfoFrameAllocator {
			free_list_loc: 0,
			phys_frame_iter: None,
			free_list_length: 0,
			memory_map: None,
			phys_memory_offset: VirtAddr::zero(),
		}
	}
	
	/// Create a FrameAllocator from the passed memory map.
	///
	/// This function is unsafe because the caller must guarantee that the passed
	/// memory map is valid. The main requirement is that all frames that are marked
	/// as `USABLE` in it are really unused.
	pub unsafe fn init(&mut self, memory_map: &'static MemoryMap, physical_memory_offset: VirtAddr) {
		self.phys_memory_offset = physical_memory_offset;
		self.phys_frame_iter = Some(Self::usable_frames(memory_map));
		self.memory_map = Some(memory_map);
	}
	
	/// Returns an iterator over the usable frames specified in the memory map.
	fn usable_frames(memory_map: &'static MemoryMap) -> PhyFrameIterator {
		let addr_ranges =
			memory_map.iter()
				.filter(|r| r.region_type == MemoryRegionType::Usable)
				.map(|r| r.range.start_addr()..r.range.end_addr());
		
		// End addr is guaranteed to be a multiple of 4096 away from start addr
		let frame_addresses =
			addr_ranges.flat_map(|r| r.step_by(4096));
		
		// create `PhysFrame` types from the start addresses
		frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
	}
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
	fn allocate_frame(&mut self) -> Option<PhysFrame> {
		if self.free_list_length > 0 {
			let ans = PhysFrame::containing_address(PhysAddr::new(self.free_list_loc));
			self.free_list_length -= 1;
			unsafe {
				self.free_list_loc = *(self.phys_memory_offset + self.free_list_loc).as_ptr::<u64>();
			}
			Some(ans)
		} else {
			self.phys_frame_iter.as_mut().unwrap().next()
		}
	}
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
	unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
		let addr = frame.start_address();
		let current_free_lst_loc = self.free_list_loc;
		self.free_list_loc = addr.as_u64();
		unsafe {
			// Prepend to our list of free blocks
			*(self.phys_memory_offset + self.free_list_loc).as_mut_ptr::<u64>() = current_free_lst_loc;
		}
		self.free_list_length += 1;
	}
}
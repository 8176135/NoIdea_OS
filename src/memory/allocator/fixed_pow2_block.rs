use core::alloc::{Layout, GlobalAlloc};
use core::{ptr, mem};
use super::Locked;
use core::ptr::NonNull;

use crate::println;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048,
	4096, 8192, 16384, 32768, 65536];

#[derive(Debug)]
struct ListNode {
	next: Option<&'static mut ListNode>,
}

#[derive(Debug)]
pub struct FixedSizeBlockAllocator {
	list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
}

impl FixedSizeBlockAllocator {
	pub const fn new() -> Self {
		FixedSizeBlockAllocator {
			list_heads: [None; BLOCK_SIZES.len()],
		}
	}
	
	/// Initialize the allocator with the given heap bounds.
	pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
		let mut current = heap_start;
		let mut current_increment_idx = BLOCK_SIZES.len() - 1;
		while {
			assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[current_increment_idx]);
			assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[current_increment_idx]);
			
			while current + BLOCK_SIZES[current_increment_idx] <= heap_start + heap_size {
				let new_node = ListNode {
					next: self.list_heads[current_increment_idx].take()
				};
				let new_node_ptr = current as *mut ListNode;
				new_node_ptr.write(new_node);
				self.list_heads[current_increment_idx] = Some(&mut *new_node_ptr);
				current += BLOCK_SIZES[current_increment_idx];
			}
			current_increment_idx -= 1;
			current_increment_idx > 0
		} {}
		println!("{:?}", self.list_heads);
	}
	
	fn split_larger_block(&mut self, block_idx: usize) -> Option<&'static mut ListNode> {
		let bigger_idx = self.pop_first_larger_block_available(block_idx)?;
		for idx in (block_idx..bigger_idx).rev() {
			// Take the bigger block
			let current_block = self.list_heads[idx + 1].take().unwrap();
			self.list_heads[idx + 1] = current_block.next.take();
			
			let next_head = self.list_heads[idx].take();
			// Split block up
			let created_block_ptr =
				((current_block as *mut ListNode as usize + BLOCK_SIZES[idx]) as *mut ListNode);
			unsafe {
				created_block_ptr.write(ListNode {
					next: next_head,
				});
				current_block.next = Some(&mut *created_block_ptr);
				self.list_heads[idx] = Some(current_block)
			}
		}
		let out = self.list_heads[block_idx].take().unwrap();
		self.list_heads[block_idx] = out.next.take();
		Some(out)
	}
	
	/// Returns Some(block_idx), or None if there are no bigger blocks available
	fn pop_first_larger_block_available(&mut self, block_idx: usize) -> Option<usize> {
		for idx in (block_idx + 1)..BLOCK_SIZES.len() {
			if self.list_heads[idx].is_some() {
				return Some(idx);
			}
		}
		None
	}
}

fn list_index(layout: &Layout) -> Option<usize> {
	let required_block_size = layout.size().max(layout.align());
	// TODO: Please optimise, though not a big deal
	BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let mut allocator = self.lock();
		match list_index(&layout) {
			Some(index) => {
				match allocator.list_heads[index].take() {
					Some(node) => {
						allocator.list_heads[index] = node.next.take();
						node as *mut ListNode as *mut u8
					}
					None => {
						allocator.split_larger_block(index)
							.map(|c| c as *mut ListNode as *mut u8)
							.unwrap_or(0 as *mut u8)
					}
				}
			}
			// TODO: Panic
			None => panic!("Allocations bigger than 2^16 not supported yet")
		}
	}
	
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let mut allocator = self.lock();
		match list_index(&layout) {
			Some(index) => {
				// Push the new node as the new head (Jumps around)
				let new_node = ListNode {
					next: allocator.list_heads[index].take(),
				};
				// verify that block has size and alignment required for storing node
				assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
				assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
				let new_node_ptr = ptr as *mut ListNode;
				new_node_ptr.write(new_node);
				allocator.list_heads[index] = Some(&mut *new_node_ptr);
			}
			None => {
				panic!("Wtf, you can't even allocate this")
			}
		}
	}
}

#[cfg(test)]
mod test {
	use crate::memory::allocator::fixed_pow2_block::BLOCK_SIZES;
	
	#[test_case]
	fn check_power_of_two() {
		let mut current = BLOCK_SIZES[0];
		for block_size in BLOCK_SIZES {
			assert_eq!(*block_size, current);
			current = current * 2;
		}
	}
}
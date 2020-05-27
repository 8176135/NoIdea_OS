use super::Result;
use super::StaticCollectionError;

pub trait Stack {
	fn get_stack_mut(&mut self) -> &mut [u64];
	fn get_size(&self) -> usize;
	fn set_size(&mut self, new_size: usize) -> Result<()>;
	
	fn clear_stack(&mut self) {
		self.set_size(0);
	}
	
	/// Returns true if the bitmap changed
	fn push(&mut self, item: u64) -> Result<()> {
		let new_idx = self.get_size();
		self.set_size(new_idx + 1)?; // Check if we are over limit
		let stack = self.get_stack_mut();
		stack[new_idx] = item;
		Ok(())
	}
	
	/// Returns true if the bitmap changed
	fn pop(&mut self) -> Result<u64> {
		if self.get_size() == 0 {
			return Err(StaticCollectionError::OutOfRange);
		}
		let item_idx = self.get_size() - 1;
		self.set_size(item_idx)?; // Check if we are past 0
		let stack = self.get_stack_mut();
		Ok(stack[item_idx])
	}
}

#[macro_export]
macro_rules! static_stack {
	($name:ident, $name_inst: ident, $size: expr) => {
		struct $name {
			stack: [u64; $size],
			size: usize,
		}
		
		impl $crate::static_collections::stack::Stack for $name {
			fn get_stack_mut(&mut self) -> &mut [u64] {
				&mut self.stack
			}
			
			fn get_size(&self) -> usize {
				self.size
			}
			
			fn set_size(&mut self, new_size: usize) -> $crate::static_collections::Result<()> {
				if new_size > $size {
					Err($crate::static_collections::StaticCollectionError::OutOfRange)
				} else {
					self.size = new_size;
					Ok(())
				}
			}
		}
		
		static $name_inst: spin::Mutex<$name> = spin::Mutex::new($name {
			stack: [0; $size],
			size: 0
		});
	}
}

#[cfg(test)]
mod test {
	use crate::static_stack;
	use crate::{serial_print, serial_println};
	use crate::static_collections::stack::Stack;
	static_stack!(TestStack, TEST_STACK, 16);
	
	#[test_case]
	fn test_stack() {
		serial_print!("test_stack... ");
		
		let mut test_lock = TEST_STACK.lock();
		test_lock.clear_stack();
		test_lock.push(1).unwrap();
		test_lock.push(2).unwrap();
		test_lock.push(3).unwrap();
		assert_eq!(test_lock.pop().unwrap(), 3);
		assert_eq!(test_lock.pop().unwrap(), 2);
		assert_eq!(test_lock.pop().unwrap(), 1);
		
		serial_println!("[ok]");
	}
}

// pub trait Bitmap {
// 	fn get_map_mut(&mut self) -> &mut [u8];
// 	fn extend_map_to_size(&mut self, size: usize) -> Result<&mut [u8], ()>;
// 	fn current_length(&self) -> Result<&mut [u8], ()>;
// }

use alloc::vec::Vec;

pub struct DynamicBitmap {
	data: Vec<u8>,
}

impl DynamicBitmap {
	pub fn new() -> DynamicBitmap {
		DynamicBitmap {
			data: Vec::new()
		}
	}
	
	/// Returns true if the bitmap changed
	pub fn set_bit(&mut self, n: usize) -> bool {
		let idx = n / 8;
		let bitset = 0x1 << (n % 8);
		
		match self.data.get_mut(idx) {
			Some(elem) if *elem & bitset == 0 => {
				*elem |= bitset;
				true
			}
			Some(elem) => false,
			None => {
				self.data.resize(idx + 1, 0);
				self.data[idx] |= bitset;
				true
			}
		}
	}
	/// Returns true if the bitmap changed
	pub fn clear_bit(&mut self, n: usize) -> bool {
		let idx = n / 8;
		let bitset = 0x1 << (n % 8);
		if self.data.get(idx).unwrap_or(&0) & bitset != 0 {
			self.data[idx] &= !bitset;
			true
		} else {
			false
		}
	}
	
	/// Returns true if bit is 1
	pub fn check_bit(&self, n: usize) -> bool {
		let idx = n / 8;
		let bitset = 0x1 << (n % 8);
		if self.data.get(idx).unwrap_or(&0) & bitset != 0 {
			true
		} else {
			false
		}
	}
	
	pub fn clear_map(&mut self) {
		for elem in &mut self.data {
			*elem = 0;
		}
	}
	
	pub fn fill_map(&mut self) {
		for elem in &mut self.data {
			*elem = u8::MAX;
		}
	}
}

#[cfg(test)]
mod test {
	use super::DynamicBitmap;
	use crate::{serial_print, serial_println};
	
	#[test_case]
	fn test_bitmap_set() {
		serial_print!("test_bitmap_set... ");
		
		let mut test_lock = DynamicBitmap::new();
		test_lock.clear_map();
		assert!(test_lock.set_bit(1));
		assert!(test_lock.set_bit(2));
		assert!(test_lock.set_bit(4));
		assert!(test_lock.set_bit(7));
		assert!(!test_lock.set_bit(4));
		assert!(!test_lock.set_bit(2));
		assert!(!test_lock.set_bit(1));
		assert!(!test_lock.set_bit(7));
		assert!(test_lock.set_bit(8));
		assert!(test_lock.set_bit(12));
		assert!(test_lock.set_bit(15));
		assert!(test_lock.set_bit(0));
		assert!(!test_lock.set_bit(8));
		assert!(!test_lock.set_bit(12));
		assert!(!test_lock.set_bit(15));
		assert!(!test_lock.set_bit(0));
		
		serial_println!("[ok]");
	}
	
	#[test_case]
	fn test_bitmap_set_clear() {
		serial_print!("test_bitmap_set_clear... ");
		
		let mut test_lock = DynamicBitmap::new();
		test_lock.clear_map();
		assert!(test_lock.set_bit(1));
		assert!(test_lock.set_bit(12));
		assert!(test_lock.set_bit(15));
		assert!(test_lock.set_bit(0));
		assert!(!test_lock.set_bit(1));
		assert!(!test_lock.set_bit(12));
		assert!(!test_lock.set_bit(15));
		assert!(!test_lock.set_bit(0));
		assert!(!test_lock.clear_bit(3));
		assert!(!test_lock.clear_bit(5));
		assert!(test_lock.clear_bit(1));
		assert!(test_lock.clear_bit(0));
		assert!(test_lock.clear_bit(12));
		assert!(test_lock.clear_bit(15));
		assert!(!test_lock.clear_bit(1));
		assert!(!test_lock.clear_bit(0));
		assert!(!test_lock.clear_bit(12));
		assert!(!test_lock.clear_bit(15));
		
		assert!(test_lock.set_bit(8));
		assert!(!test_lock.set_bit(8));
		
		assert!(test_lock.set_bit(1));
		assert!(test_lock.set_bit(12));
		assert!(test_lock.set_bit(15));
		assert!(test_lock.set_bit(0));
		
		serial_println!("[ok]");
	}
}

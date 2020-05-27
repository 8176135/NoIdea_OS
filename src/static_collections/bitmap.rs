use super::Result;
use super::StaticCollectionError;

pub trait Bitmap {
	fn get_map_mut(&mut self) -> &mut [u8];
	
	fn clear_map(&mut self) {
		for elem in self.get_map_mut() {
			*elem = 0;
		}
	}
	
	fn fill_map(&mut self) {
		for elem in self.get_map_mut() {
			*elem = u8::MAX;
		}
	}
	
	/// Returns true if the bitmap changed
	fn set_bit(&mut self, n: usize) -> Result<bool> {
		let map = self.get_map_mut();
		let idx = n / 8;
		let bitset = 0x1 << (n % 8);
		if map.get(idx).ok_or(StaticCollectionError::OutOfRange)? & bitset == 0 {
			map[idx] |= bitset;
			Ok(true)
		} else {
			Ok(false)
		}
	}
	/// Returns true if the bitmap changed
	fn clear_bit(&mut self, n: usize) -> Result<bool> {
		let map = self.get_map_mut();
		let idx = n / 8;
		let bitset = 0x1 << (n % 8);
		if map.get(idx).ok_or(StaticCollectionError::OutOfRange)? & bitset != 0 {
			map[idx] &= !bitset;
			Ok(true)
		} else {
			Ok(false)
		}
	}
}

#[macro_export]
macro_rules! static_bitmap {
	($name:ident, $name_inst: ident, $size: expr) => {
		struct $name {
			map: [u8; ($size + 7) / 8],
		}
		
		impl $crate::static_collections::bitmap::Bitmap for $name {
			fn get_map_mut(&mut self) -> &mut [u8] {
				&mut self.map
			}
		}
		
		static $name_inst: spin::Mutex<$name> = spin::Mutex::new($name {
			map: [0; ($size + 7) / 8]
		});
	}
}

#[cfg(test)]
mod test {
	use crate::static_bitmap;
	use crate::{serial_print, serial_println};
	use crate::static_collections::bitmap::Bitmap;
	static_bitmap!(TestBitmap, TEST_BITMAP, 16);
	
	#[test_case]
	fn test_bitmap_set() {
		serial_print!("test_bitmap_set... ");
		
		let mut test_lock = TEST_BITMAP.lock();
		test_lock.clear_map();
		assert!(   test_lock.set_bit(1).unwrap());
		assert!(   test_lock.set_bit(2).unwrap());
		assert!(   test_lock.set_bit(4).unwrap());
		assert!(   test_lock.set_bit(7).unwrap());
		assert!( ! test_lock.set_bit(4).unwrap());
		assert!( ! test_lock.set_bit(2).unwrap());
		assert!( ! test_lock.set_bit(1).unwrap());
		assert!( ! test_lock.set_bit(7).unwrap());
		assert!(   test_lock.set_bit(8).unwrap());
		assert!(   test_lock.set_bit(12).unwrap());
		assert!(   test_lock.set_bit(15).unwrap());
		assert!(   test_lock.set_bit(0).unwrap());
		assert!( ! test_lock.set_bit(8).unwrap());
		assert!( ! test_lock.set_bit(12).unwrap());
		assert!( ! test_lock.set_bit(15).unwrap());
		assert!( ! test_lock.set_bit(0).unwrap());
		
		serial_println!("[ok]");
	}
	
	#[test_case]
	fn test_bitmap_set_clear() {
		serial_print!("test_bitmap_set_clear... ");
		
		let mut test_lock = TEST_BITMAP.lock();
		test_lock.clear_map();
		assert!(   test_lock.set_bit(1).unwrap());
		assert!(   test_lock.set_bit(12).unwrap());
		assert!(   test_lock.set_bit(15).unwrap());
		assert!(   test_lock.set_bit(0).unwrap());
		assert!( ! test_lock.set_bit(1).unwrap());
		assert!( ! test_lock.set_bit(12).unwrap());
		assert!( ! test_lock.set_bit(15).unwrap());
		assert!( ! test_lock.set_bit(0).unwrap());
		assert!( ! test_lock.clear_bit(3).unwrap());
		assert!( ! test_lock.clear_bit(5).unwrap());
		assert!(   test_lock.clear_bit(1).unwrap());
		assert!(   test_lock.clear_bit(0).unwrap());
		assert!(   test_lock.clear_bit(12).unwrap());
		assert!(   test_lock.clear_bit(15).unwrap());
		assert!( ! test_lock.clear_bit(1).unwrap());
		assert!( ! test_lock.clear_bit(0).unwrap());
		assert!( ! test_lock.clear_bit(12).unwrap());
		assert!( ! test_lock.clear_bit(15).unwrap());
		
		assert!(   test_lock.set_bit(8).unwrap());
		assert!( ! test_lock.set_bit(8).unwrap());
		
		assert!(   test_lock.set_bit(1).unwrap());
		assert!(   test_lock.set_bit(12).unwrap());
		assert!(   test_lock.set_bit(15).unwrap());
		assert!(   test_lock.set_bit(0).unwrap());
		
		serial_println!("[ok]");
	}
}

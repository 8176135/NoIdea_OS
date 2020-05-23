use super::Result;
use super::StaticCollectionError;

trait Bitmap {
	fn get_map(&mut self) -> &mut [u8];
	
	/// Returns true if the bitmap changed
	fn set_bit(&mut self, n: usize) -> Result<bool> {
		let map = self.get_map();
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
		let map = self.get_map();
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
pub mod bitmap;
pub mod stack;

type Result<T> = core::result::Result<T, StaticCollectionError>;

enum StaticCollectionError {
	OutOfRange,
}

use core::fmt;

impl fmt::Display for StaticCollectionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			StaticCollectionError::OutOfRange => writeln!(f, "Out of index range"),
		}
	}
}

macro_rules! bitmap {
	($name:stmt, $size: expr) => {
		struct $stmt {
			map: [u8; $size] = [0; $size],
		}
		
		impl bitmap::Bitmap for $stmt {
			fn get_map(&mut self) -> &mut [u8] {
				&mut self.map
			}
		}
	}
}
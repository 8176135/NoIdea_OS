pub mod bitmap;
pub mod stack;

pub type Result<T> = core::result::Result<T, StaticCollectionError>;

#[derive(Debug, Copy, Clone)]
pub enum StaticCollectionError {
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
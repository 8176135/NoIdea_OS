use alloc::vec::Vec;

pub struct IncrementingPool {
	pool: Vec<u64>,
	current_max: u64,
}

impl IncrementingPool {
	pub fn new(staring_point: u64) -> IncrementingPool {
		IncrementingPool {
			pool: Vec::new(),
			current_max: staring_point
		}
	}
	
	pub fn get_free_elem(&mut self) -> u64 {
		if let Some(out) = self.pool.pop() {
			out
		} else {
			self.current_max += 1;
			self.current_max - 1
		}
	}
	
	pub fn return_elem(&mut self, elem: u64) {
		self.pool.push(elem);
	}
}
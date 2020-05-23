use super::Result;
use super::StaticCollectionError;

trait Stack {
	fn get_stack(&mut self) -> &mut [u8];
	fn current_idx(&self) -> usize;
	fn increment_idx(&mut self) -> Result<usize>;
	fn decrement_idx(&mut self) -> Result<usize>;
	
	/// Returns true if the bitmap changed
	fn push(&mut self, item: u8) -> Result<()> {
		let new_idx = self.increment_idx()?;
		let stack = self.get_stack();
		stack[new_idx] = item;
		Ok(())
	}
	
	/// Returns true if the bitmap changed
	fn pop(&mut self, item: u8) -> Result<u8> {
		let idx = self.current_idx();
		let stack = self.get_stack();
		let ans = stack[idx];
		self.decrement_idx()?;
		Ok(ans)
	}
}
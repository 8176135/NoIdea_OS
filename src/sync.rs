#[derive(Debug)]
pub struct Semaphore {
	count: usize,
	// sleepers: []
}

impl Semaphore {}

type Mutex = Semaphore;
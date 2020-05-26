use crate::histogram::Histogram;

// LIVENESS

pub struct LivenessCounter {
	n: usize, 	// Timer
	m: usize,	// Number of objects
	alloc_sum: Histogram,	// Sum of allocation times before time
	alloc_counts: Histogram,	// Number of allocations before time
	free_sum: Histogram,	// Sum of free times before time
	free_counts: Histogram,		// Number of frees before time
}

impl LivenessCounter {
	pub fn new() -> LivenessCounter {
		LivenessCounter {
			n: 1,		// Start at 1 or 0?
			m: 0,
			alloc_sum: Histogram::new(),		// Need to add anything at start?
			alloc_counts: Histogram::new(),
			free_sum: Histogram::new(),
			free_counts: Histogram::new()
		}
	}

	pub fn alloc(&mut self) -> () {
		self.alloc_sum.add(self.n, self.n);
		self.alloc_counts.increment(self.n);
		self.m += 1;
	}

	pub fn free(&mut self) -> () {
		self.free_sum.add(self.n, self.n);
		self.free_counts.increment(self.n);
	}

	pub fn inc_timer(&mut self) -> () {
		self.n += 1;
		self.alloc_counts.add(self.n, self.alloc_counts.get(&(self.n-1)));
		self.alloc_sum.add(self.n, self.alloc_sum.get(&(self.n-1)));
		self.free_counts.add(self.n, self.free_counts.get(&(self.n-1)));
		self.free_sum.add(self.n, self.free_sum.get(&(self.n-1)));
	}

	pub fn liveness(&self, k: usize) -> usize {
		let i = self.n-k+1;
		let tmp1 = (self.m-self.free_counts.get(&i)) * i + self.free_sum.get(&i);
		let tmp2 = self.alloc_counts.get(&k) * k + self.alloc_sum.get(&self.n) - self.alloc_sum.get(&k);
		(tmp1 - tmp2 + self.m * k) / i
	}
}

// REUSE

pub struct ReuseCounter {
	burst_length: usize,
	hibernation_period: usize
}
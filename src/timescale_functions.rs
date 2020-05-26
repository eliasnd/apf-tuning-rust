use std::vec::Vec;
use std::cmp::min;
use std::cmp::max;

use crate::trace::*;
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

	pub fn alloc(&mut self) {
		self.alloc_sum.add(self.n, self.n);
		self.alloc_counts.increment(self.n);
		self.m += 1;
	}

	pub fn free(&mut self) {
		self.free_sum.add(self.n, self.n);
		self.free_counts.increment(self.n);
	}

	pub fn inc_timer(&mut self) {
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
	burst_length: usize,		// Length of bursts
	hibernation_period: usize,	// Length of hibernation
	n: usize,					// Current time counter
	trace: Option<Trace>,		// Optional current trace -- none if hibernating	
	reuse: usize				// Last calculated reuse -- 0 if not initialized (?)
}

impl ReuseCounter {
	pub fn new(bl: usize, hp: usize) -> ReuseCounter {
		ReuseCounter {
			burst_length: bl,
			hibernation_period: hp,
			n: 0,
			trace: Some(Trace::new()),	// Start sampling or hibernating?
			reuse: 0
		}
	}

	pub fn alloc(&mut self, slot: usize) {
		match &mut self.trace {
			Some(t) => { t.add(Event::Alloc(slot)); }
			None => {}
		}
	}

	pub fn free(&mut self, slot: usize) {
		match &mut self.trace {
			Some(t) => { t.add(Event::Free(slot)); }
			None => {}
		}
	}

	// Maybe test if sampling before calling alloc and free? Not sure
	pub fn sampling(&self) -> bool {
		self.trace.is_some()
	}

	pub fn inc_timer(&mut self) -> () {
		self.n += 1;
		match &self.trace.is_some() {
			true => {
				if self.n > self.burst_length {
					self.n = 0;
					self.trace = None;
					// Calculate reuse
				}
			}
			false => {
				if self.n > self.hibernation_period {
					self.n = 0;
					self.trace = Some(Trace::new());
				}
			}
		}
	}
}

// Offline functions

// Reuse calculates all k and returns histogram indexed by k
// Note: Returns histogram is NOT RI or RD Histogram
fn reuse(t: Trace) -> Histogram {
	let intervals = trace_to_free_intervals(&t);
	let n = t.length();

	let mut x = vec![0; n];	// X(i) = x[i-1]
	let mut y = vec![0; n];	// Y(i) = y[i-1]
	let mut z = vec![0; n];	// Z(i) = z[i-1]

	// Base Case -- construct X(1), Y(1), Z(1)
	let (mut x0, mut y0, mut z0) = (0, 0, 0);
	
	for i in 0..intervals.len() {
		let interval = *intervals.get(i).unwrap();	// Safe since only looping over range
		if interval.1 - interval.0 == 1 { 
			x0 += interval.0; 
			y0 += interval.1;
			z0 += 2;
		}
	}

	x[0] = x0;
	y[0] = y0;
	z[0] = z0;

	// Recursive Case
	for i in 1..t.length() {
		let k = i+1;
		let (mut xk, mut yk, mut zk) = (0, 0, 0);	// These represent values to be added to previous index

		for i in 0..intervals.len() {
			let interval = *intervals.get(i).unwrap();

			// X(k)
			if interval.0 >= n-(k-1) { xk -= 1; }
			if interval.1 - interval.0 == k { xk += min(n-k, interval.0); }

			// Y(k)
			if interval.1 <= k-1 { yk += 1; }
			if interval.1 - interval.0 == k { yk += max(k, interval.1); }	// Redoing conditional for readability for now

			// Z(k)
			if interval.1 - interval.0 <= k { zk += 1; }
			if interval.1 - interval.0 == k { zk += k; }
		}

		x[i] = xk;
		y[i] = yk;
		z[i] = zk;
	}

	// Construct histogram
	let mut result = Histogram::new();

	for k in 1..n+1 { result.add(k, (x[k-1] + y[k-1] + z[k-1]) / (n-k+1)); }

	result
}
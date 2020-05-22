use std::collections::HashMap;

struct Histogram
{
	histogram: HashMap<usize, u16>
}

impl Histogram
{
	pub fn new() -> Histogram {
		Histogram {
			histogram: HashMap::new()
		}
	}

	pub fn increment(&mut self, key: usize) -> () {
		*self.histogram.entry(key).or_insert(0) += 1;
	}

	pub fn get(&self, key: &usize) -> u16 {
		match self.histogram.get(key) {
			Some(n) => *n,
			None => 0
		}
	}
}
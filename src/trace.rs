struct Trace 
{
	accesses: Vec<usize>,
	length: usize
}

impl Trace
{
	pub fn new() -> Trace {
		Trace {
			accesses: Vec::new(),
			length: 0
		}
	}

	pub fn length(&self) -> usize {
		self.length
	}

	pub fn add(&mut self, add: usize) -> () {
		self.accesses.push(add);
		self.length += 1;
	}

	pub fn get(&self, index: usize) -> usize {
		self.accesses[index]
	}
}
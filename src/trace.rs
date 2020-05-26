#[derive(Copy, Clone)]
pub enum Event {
    Alloc(usize),
    Free(usize)
}

pub struct Trace {
    accesses: Vec<Event>,
    length: usize,
}

impl Trace {
    pub fn new() -> Trace {
        Trace {
            accesses: Vec::new(),
            length: 0,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn add(&mut self, add: Event) -> () {
        self.accesses.push(add);
        self.length += 1;
    }

    pub fn get(&self, index: usize) -> Event {
        self.accesses[index]
    }

    pub fn sub(&self, start: usize, end: usize) -> Trace {
        let mut t = Trace::new();
        for i in start..end {
            t.add(self.get(i));
        }
        t
    }
}

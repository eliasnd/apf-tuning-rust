use std::collections::HashMap;
use std::vec::Vec;

use crate::trace::Event::*;

/*
    Event represents allocation or free operation
    usize stores heap slot -- not sure how helpful this will be in practice, so might make it generic
*/
#[derive(Copy, Clone)]
pub enum Event {
    Alloc(usize),
    Free(usize)
}

pub struct Trace {
    accesses: Vec<Event>,
    length: usize,
}


/*
    Memory trace
    Simple wrapper for vector of events
*/
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

    // Counts objects referenced in trace
    pub fn object_count(&self) -> usize {   // This is dumb
        let mut seen = HashMap::new();

        for i in 0..self.length() {
            match &self.get(i) {
                Alloc(s) => { if !seen.contains_key(s) { seen.insert(s.clone(), true); } },
                Free(s) => { if !seen.contains_key(s) { seen.insert(s.clone(), true); } }
            };
            
        }

        seen.len()
    }

    // Generates a subtrace from start to end, excluding end
    // Returns None if indices not valid
    pub fn subtrace(&self, start: usize, end: usize) -> Option<Trace> {
        if start > end { return None; }
        if end > self.length() { return None; }

        let mut t = Trace::new();
        for i in start..end {
            t.add(self.get(i));
        }
        Some(t)
    }
}

// Converts trace to vector of free intervals represented (si, ei)
pub fn trace_to_free_intervals(t: &Trace) -> Vec<(usize, usize)> {
    let mut allocs = HashMap::<usize, usize>::new();
    let mut result = Vec::new();

    for i in 0..t.length() {
        match t.get(i) {
            Alloc(s) => { allocs.insert(s.clone(), i); },
            Free(s) => { result.push((*allocs.get(&s).expect("Free before alloc"), i)); }   // Should format error to include index
        }
    }

    result
}

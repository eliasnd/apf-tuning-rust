use crate::constants::INIT_TRACE_LENGTH;
use std::collections::HashMap;
use std::fmt;
use std::vec::Vec;

/*
    Event represents allocation or free operation
    usize stores heap slot -- not sure how helpful this will be in practice, so might make it generic
*/
#[derive(Copy, Clone)]
pub enum Event {
    Alloc(usize),
    Free(usize),
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alloc(u) => write!(f, "a{}", u),
            Free(u) => write!(f, "f{}", u),
        }
    }
}

use crate::trace::Event::*;
use std::ffi::c_void;
use std::mem::size_of;

// Need trace implementation that doesn't call alloc
#[derive(Debug)]
pub struct Trace {
    accesses: Vec::<Event>,
    length: usize,
    alloc_count: usize,
}

/*
    Memory trace
    Simple wrapper for vector of events
*/
impl Trace {
    pub fn new() -> Trace {
        Trace {
            accesses: Vec::with_capacity(INIT_TRACE_LENGTH),
            length: 0,
            alloc_count: 0,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn alloc_length(&self) -> usize {
        self.alloc_count
    }

    pub fn add(&mut self, add: Event) -> () {
        self.accesses.push(add);
        self.length += 1;
        match add {
            Alloc(_) => {
                self.alloc_count += 1;
            }
            Free(_) => {}
        };
    }

    pub fn get(&self, index: usize) -> Event {
        self.accesses[index]
    }

    // Counts objects referenced in trace
    pub fn object_count(&self) -> usize {
        // This is dumb
        let mut seen = HashMap::new();

        for i in 0..self.length() {
            match &self.get(i) {
                Alloc(s) => {
                    if !seen.contains_key(s) {
                        seen.insert(s.clone(), true);
                    }
                }
                Free(s) => {
                    if !seen.contains_key(s) {
                        seen.insert(s.clone(), true);
                    }
                }
            };
        }

        seen.len()
    }

    // Converts trace to vector of free intervals represented (si, ei)
    pub fn free_intervals(&self) -> Vec<(usize, usize)> {
        let mut frees = HashMap::<usize, usize>::new();
        let mut result = Vec::new();

        let mut alloc_clock = 0;

        for i in 0..self.length() {
            match self.get(i) {
                Free(s) => {
                    frees.insert(s.clone(), alloc_clock);
                }
                Alloc(e) => {
                    match frees.get(&e) {
                        Some(&s) => {
                            result.push((s, alloc_clock));
                        } // Should format error to include index
                        None => {}
                    }
                    alloc_clock += 1;
                }
            }
        }

        result
    }

    // Converts tract to vector of free intervals represented by (s_i, e_i)
    // Does not use allocation clock
    pub fn free_intervals_alt(&self) -> Vec<(usize, usize)> {
        let mut frees = HashMap::<usize, usize>::new();
        let mut result = Vec::new();

        for i in 0..self.length() {
            match self.get(i) {
                Free(s) => {
                    frees.insert(s.clone(), i);
                }
                Alloc(e) => {
                    match frees.get(&e) {
                        Some(&s) => {
                            result.push((s, i));
                        } // Should format error to include index
                        None => {}
                    }
                }
            }
        }

        result
    }

    // Check validity of trace -- might be useful later
    pub fn valid(&self) -> bool {
        let mut alloc = HashMap::<usize, bool>::new();

        for i in 0..self.length() {
            match self.get(i) {
                Alloc(s) => {
                    match alloc.insert(s, true) {
                        Some(b) => {
                            if b == true {
                                return false;
                            }
                        } // If already allocated, fail
                        _ => {}
                    }
                }
                Free(s) => {
                    match alloc.insert(s, false) {
                        Some(b) => {
                            if b == false {
                                return false;
                            }
                        } // If already freed, fail
                        _ => {
                            return false;
                        } // If never allocated, fail
                    }
                }
            }
        }

        return true;
    }
}

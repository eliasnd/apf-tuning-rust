use std::collections::HashMap;
use std::vec::Vec;

/*
    Event represents allocation or free operation
    usize stores heap slot -- not sure how helpful this will be in practice, so might make it generic
*/
#[derive(Copy, Clone)]
pub enum Event {
    Alloc(usize),
    Free(usize)
}

use crate::trace::Event::*;

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

    pub fn extend(&mut self, vec: Vec<Event>) -> () {
        self.accesses.append(&mut vec.clone());
        self.length += vec.len();
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

    // Converts trace to vector of free intervals represented (si, ei)
    pub fn free_intervals(&self) -> Vec<(usize, usize)> {
        let mut frees = HashMap::<usize, usize>::new();
        let mut result = Vec::new();

        for i in 0..self.length() {
            match self.get(i) {
                Free(s) => { frees.insert(s.clone(), i); },
                Alloc(e) => { match frees.get(&e) {
                                Some(&s) => { result.push((s, i)); }   // Should format error to include index
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
                Alloc(s) => { match alloc.insert(s, true) {
                                Some(b) => { if b == true { return false; } }   // If already allocated, fail
                                _ => {}
                              } 
                }
                Free(s) => { match alloc.insert(s, false) {
                                Some(b) => { if b == false { return false; } }   // If already freed, fail
                                _ => { return false; }  // If never allocated, fail
                            }
                }
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(1), Alloc(2), Free(1)]);
        assert_eq!(t.length(), 3);
    }

   /* #[test]
    fn test_length_2() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(3), Free(3), Alloc(3), Free(3)]);
        assert_eq!(t.length(), 4);
    } */

    #[test]
    fn test_obj_count() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(1), Alloc(2), Alloc(4), Free(1)]);
        assert_eq!(t.object_count(), 3);
    }

   /*  #[test]
    fn test_obj_count_2() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(3), Free(3), Alloc(3), Free(3)]);
        assert_eq!(t.object_count(), 1);
    } */

    #[test]
    fn test_valid() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(1), Alloc(2), Free(1), Free(2), Alloc(5), Free(5)]);
        assert_eq!(t.valid(), true);
    }

    #[test]
    fn test_invalid() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(3), Free(3), Alloc(3), Free(3), Free(3)]);
        assert_eq!(t.valid(), false);
    }

    #[test]
    fn test_intervals() {
        let mut t = Trace::new();
        t.extend(vec![Free(1), Free(3), Alloc(3), Alloc(1), Free(2), Free(3), Free(1), Alloc(2), Alloc(1), Alloc(3)]);
        assert_eq!(t.free_intervals(), vec![(1, 2), (0, 3), (4, 7), (6, 8), (5, 9)]);
    }

    #[test]
    fn test_intervals_2() {
        let mut t = Trace::new();
        t.extend(vec![Alloc(1), Alloc(2), Alloc(3), Free(3), Free(2), Free(1), 
                      Alloc(1), Alloc(2), Alloc(3), Free(3), Free(2), Free(1),
                      Alloc(1), Alloc(2), Alloc(3)]);
        assert_eq!(t.free_intervals(), vec![(5, 6), (4, 7), (3, 8), (11, 12), (10, 13), (9, 14)]);
    }
}

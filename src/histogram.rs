use crate::constants::INIT_HISTOGRAM_LENGTH;
use std::vec::Vec;

/*
    Histogram class -- really just a Hashmap
*/
#[derive(Debug)]
pub struct Histogram {
    histogram: Vec::<usize>,
}

impl Histogram {
    pub fn new() -> Histogram {
        Histogram {
            histogram: Vec::with_capacity(INIT_HISTOGRAM_LENGTH),
        }
    }

    pub fn increment(&mut self, key: usize) -> () {
        if key >= self.histogram.capacity() - 1 {
            self.grow(key);
        }

        self.histogram[key] += 1;
    }

    pub fn add(&mut self, key: usize, val: usize) {
        if key >= self.histogram.capacity() - 1 {
            self.grow(key);
        }

        self.histogram[key] = self.histogram[key] + val;
        // unsafe { (&mut self.histogram[key]as *mut usize).write(self.histogram[key] + val) };
    }

    pub fn get(&self, key: usize) -> usize {
        if key >= self.histogram.capacity() {
            return 0;
        }
        self.histogram[key]
    }

    // Returns number of keys
    pub fn count(&self) -> usize {
        self.histogram.len()
    }

    pub fn grow(&mut self, failed_key: usize) {
        let oldMax = self.histogram.capacity();

        let mut reserve_amount = oldMax;
        while reserve_amount <= failed_key {
            reserve_amount *= 2;
        }

        self.histogram.reserve(reserve_amount);

        for i in oldMax..self.histogram.capacity() {
            self.histogram[i] = 0;
        }
    }
}

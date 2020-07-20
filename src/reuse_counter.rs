use crate::trace::*;

use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;

/*
    Reuse Counter
    Again, call alloc() and free() whenever needed
    To check if counter is currently in a burst, try sampling()
    inc_timer() works as described for liveness
    reuse(k) gets reuse for windows of length k
*/

#[derive(Debug)]
pub struct ReuseCounter {
    burst_length: usize,                // Length of bursts
    hibernation_period: usize,          // Length of hibernation
    n: usize,                           // Current time counter
    trace: Option<Trace>,           // Optional current trace -- none if hibernating
    reuse: Option<HashMap<usize, f32>>, // Last calculated reuse -- none if not initialized (?)
}

impl ReuseCounter {
    pub fn new(bl: usize, hp: usize) -> ReuseCounter {
        ReuseCounter {
            burst_length: bl,
            hibernation_period: hp,
            n: 0,
            trace: Some(Trace::new()), // Start sampling or hibernating?
            reuse: None,
        }
    }

    pub fn alloc(&mut self, slot: usize) {
        match &mut self.trace {
            Some(t) => {
                t.add(Event::Alloc(slot));
            }
            None => {}
        }
    }

    pub fn free(&mut self, slot: usize) {
        match &mut self.trace {
            Some(t) => {
                t.add(Event::Free(slot));
            }
            None => {}
        }
    }

    // Maybe test if sampling before calling alloc and free? Not sure
    pub fn sampling(&self) -> bool {
        self.trace.is_some()
    }

    pub fn inc_timer(&mut self) -> () {
        self.n += 1;
        match &self.trace {
            Some(trace) => {
                if self.n >= self.burst_length {
                    self.reuse = Some(reuse(trace));
                    self.n = 0;
                    self.trace = None;
                }
            }
            None => {
                if self.n >= self.hibernation_period {
                    self.n = 0;
                    self.trace = Some(Trace::new());
                }
            }
        }
    }

    pub fn reuse(&self, k: usize) -> Option<f32> {
        // if k > self.burst_length { panic!("ERROR: k exceeds burst length"); }
        match &self.reuse {
            Some(reuse) => match reuse.get(&k) {
                Some(n) => Some(*n),
                None => Some(0.0),
            },
            None => None,
        }
    }
}

// Offline Functions

fn reuse(t: &Trace) -> HashMap<usize, f32> {
    let intervals = t.free_intervals();
    let n = t.alloc_length();

    // Predicate terms
    let mut start_index_counts = vec![0; n]; // s_i
    let mut end_index_counts = vec![0; n]; // e_i
    let mut len_counts = vec![0; n]; // e_i - s_i -- indices decremented by 1 since no len 0
    let mut start_indices_sums = vec![0; n]; // I(e_i - s_i = k) * s_i -- indices decremented by 1
    let mut start_indices_min_sums = vec![0; n]; // I(e_i - s_i = k) * min(n-k, s_i) -- indices decremented by 1
    let mut end_indices_sums = vec![0; n]; // I(e_i - s_i = k) * e_i -- indices decremented by 1
    let mut end_indices_max_sums = vec![0; n]; // I(e_i - s_i = k) * max(k, e_i) -- indices decremented by 1

    for i in 0..intervals.len() {
        let interval = intervals[i];
        let len = interval.1 - interval.0 + 1;

        start_index_counts[interval.0] += 1;
        end_index_counts[interval.1] += 1;
        len_counts[len - 1] += 1;
        start_indices_sums[len - 1] += interval.0;
        start_indices_min_sums[len - 1] += min(n - len, interval.0);
        end_indices_sums[len - 1] += interval.1;
        end_indices_max_sums[len - 1] += max(len, interval.1);
    }

    let mut start_index_n_k = vec![0; n]; // I(s_i >= (n-k))
    let mut end_index_k_1 = vec![0; n]; // I(e_i <= k-1)
    let mut len_l_k = vec![0; n]; // I(e_i - s_i <= k)

    start_index_n_k[0] = 0; // Cannot start at index n -- remember index 0 -> k = 1
    end_index_k_1[0] = 0; // Cannot end at index 0
    len_l_k[0] = len_counts[0]; // I(e_i - s_i <= 1) = I(e_i - s_i = 1)

    for i in 1..n {
        start_index_n_k[i] = start_index_n_k[i - 1] + start_index_counts[n - i];
        end_index_k_1[i] = end_index_k_1[i - 1] + end_index_counts[i];
        len_l_k[i] = len_l_k[i - 1] + len_counts[i];
    }
    let mut x = vec![0; n]; // X(i) = x[i-1]
    let mut y = vec![0; n]; // Y(i) = y[i-1]
    let mut z = vec![0; n]; // Z(i) = z[i-1]

    x[0] = start_indices_sums[0];
    y[0] = end_indices_sums[0];
    z[0] = len_counts[0];

    for i in 1..n {
        let k = i + 1;

        x[i] = x[i - 1] + start_indices_min_sums[i] - start_index_n_k[i];
        y[i] = y[i - 1] + end_index_k_1[i - 1] + end_indices_max_sums[i];
        z[i] = z[i - 1] + len_l_k[i - 1] + k * len_counts[i];
    }

    let mut result = HashMap::<usize, f32>::new();
    for k in 1..n + 1 {
        result.insert(
            k,
            (x[k - 1] + z[k - 1] - y[k - 1]) as f32 / (n - k + 1) as f32,
        );
    }

    result
}

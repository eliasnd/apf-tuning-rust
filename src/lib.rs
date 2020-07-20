use crate::liveness_counter::LivenessCounter;
use crate::reuse_counter::ReuseCounter;

use crate::constants::{REUSE_BURST_LENGTH, REUSE_HIBERNATION_PERIOD, USE_ALLOCATION_CLOCK};

mod histogram;
mod liveness_counter;
mod reuse_counter;
mod trace;
mod constants;

static mut TARGET_APF: Option<usize> = None;

pub fn set_target_apf(tapf: usize) {
    unsafe {
        TARGET_APF = match TARGET_APF {
            Some(t) => { panic!("ERROR: Target apf already set"); },
            None => Some(tapf)
        };
    }
}

/*
        -- APF Tuner --
    * One for each size container
    * Call malloc() and free() whenever those operations are performed
*/
pub struct ApfTuner {
    id: usize,
    l_counter: LivenessCounter,
    r_counter: ReuseCounter,
    time: usize,
    fetch_count: usize,
    check: fn(usize) -> u32,
    get: fn(usize, usize) -> bool,
    ret: fn(usize, u32) -> bool,
}

impl ApfTuner {
    pub fn new(
        id: usize,
        check: fn(usize) -> u32,
        get: fn(usize, usize) -> bool,
        ret: fn(usize, u32) -> bool
    ) -> ApfTuner {
        unsafe {
            match TARGET_APF {
                Some(t) => {}
                None => { panic!("ERROR: Trying to construct ApfTuner with uninitialized Target apf"); }
            }
        }
        
        ApfTuner {
            id,
            l_counter: LivenessCounter::new(),
            r_counter: ReuseCounter::new(REUSE_BURST_LENGTH, REUSE_HIBERNATION_PERIOD),
            time: 0,
            fetch_count: 0,
            check,
            get,
            ret
        }
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn malloc(&mut self, ptr: *mut u8) -> bool {
        self.time += 1;

        if !USE_ALLOCATION_CLOCK {
            self.l_counter.inc_timer();
            self.l_counter.alloc();
        }

        self.r_counter.alloc(ptr as usize);
        self.r_counter.inc_timer();

        // If out of free blocks, fetch
        if (self.check)(self.id) == 0 {
            let demand;

            match self.demand(self.calculate_dapf().into()) {
                Some(d) => {
                    demand = d;
                }
                None => {
                    return false;
                }
            }

            (self.get)(self.id, demand.ceil() as usize);
        }

        return true;
    }

    // Processes free event.
    // Check function returns number of available slots
    // Ret function returns number of slots to central reserve
    // Returns true if demand can be calculated (reuse counter has completed a burst), false if not
    pub fn free(&mut self, ptr: *mut u8) -> bool {
        self.r_counter.free(ptr as usize);
        if !USE_ALLOCATION_CLOCK {
            self.r_counter.inc_timer();
            self.time += 1;
            self.l_counter.inc_timer();
            self.l_counter.free();
        }

        let d = self.demand(self.calculate_dapf().into());

        if d.is_none() || d.unwrap() < 0.0 {
            return false;
        }
        let demand = d.unwrap(); // Safe

        // If too many free blocks, return some
        if (self.check)(self.id) as f32 >= 2.0 * demand + 1.0 {
            let ceil = demand.ceil() as u32;
            (self.ret)(self.id, ceil + 1);
        }
        true
    }

    fn count_fetch(&mut self) {
        self.fetch_count += 1;
    }

    fn calculate_dapf(&self) -> usize {
        unsafe {
            if self.time >= TARGET_APF.unwrap() * (self.fetch_count + 1) {
                TARGET_APF.unwrap()
            } else {
                TARGET_APF.unwrap() * (self.fetch_count + 1) - self.time
            }
        }
    }

    // Average demand in windows of length k
    // Returns none if reuse counter has not completed a burst yet
    fn demand(&self, k: usize) -> Option<f32> {
        if k > self.time {
            return None;
        }

        match self.r_counter.reuse(k) {
            Some(r) => {
                if USE_ALLOCATION_CLOCK {
                    Some(k as f32 - r)
                } else {
                    Some(self.l_counter.liveness(k) - self.l_counter.liveness(0) - r)
                }
            }
            None => None,
        }
    }
}

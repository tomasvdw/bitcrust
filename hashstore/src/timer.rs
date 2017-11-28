

use std::time;
use std::sync::atomic;

/// Struct for maintaining timings of operations
pub struct Timer {
    start: time::Instant,
    stat: &'static atomic::AtomicU64
}

impl Timer {
    pub fn new(stat: &'static atomic::AtomicU64) -> Self {
        Timer {
            start: time::Instant::now(),
            stat: stat
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        let elapsed = (elapsed.as_secs() as u64 * 1_000_000_000)
            + elapsed.subsec_nanos() as u64;
        self.stat.fetch_add(elapsed, atomic::Ordering::Relaxed);
    }
}

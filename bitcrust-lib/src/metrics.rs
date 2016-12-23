//! Small metrics library


use std::collections::HashMap;
use std::time::{Instant,Duration};


use std::sync::Arc;
use std::cell::RefCell;

/// Represents a single named countable value
struct Metric {
    time:   Duration,
    count:  usize,
    ticker: usize
}

/// A handle to a timed key; this adds itself to the referenced metric
/// when going out-of-scope
pub struct RunningMetric<'a>{
    metric:     &'a Metrics,
    name:       &'static str,
    started:    Instant,
    ticker:     usize,
    is_stopped: bool,
}

pub struct Metrics {

    metrics: Arc<RefCell<HashMap<&'static str, Metric>>>

}



impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            metrics: Arc::new(RefCell::new(HashMap::new()))
        }
    }

    /// Start measuring time at the given tag name.
    /// Stops when the result go out of scope
    ///
    /// Hence the results must be saved in a temporary variable. Use a _ prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use bitcrust_lib::metrics;
    ///
    /// let metrics = metrics::Metrics::new();
    /// let _m = metrics.start("mymetric");
    /// ```
    ///
    pub fn start(&self, name: &'static str) -> RunningMetric {

        RunningMetric {
            started:    Instant::now(),
            metric:     self,
            name:       name,
            ticker:     0,
            is_stopped: false

        }
    }
}


impl Drop for Metrics {

    /// We dump everything on exit
    fn drop(&mut self) {

        let metrics = self.metrics.borrow();
        println!("METRICS:");

        for k in metrics.keys() {

            let v = metrics.get(k).unwrap();


            if v.count == 0 {
                continue;
            }

            let micros: usize = v.time.as_secs() as usize * 1000_000 + v.time.subsec_nanos() as usize / 1_000 as usize;
            println!("{}", k);


            println!("  count={}", v.count);
            println!("  durat={}", micros);
            println!("    avg={}", micros / v.count);
            println!("  tickr={}", v.ticker);
            println!("    avg={}", v.ticker / v.count);


        }
    }
}

impl<'a> RunningMetric<'a> {

    // Close this metric and add the numbers to the main metrics table
    fn stop(&mut self) {

        if self.is_stopped {
            return;
        }

        let mut metrics = self.metric.metrics.borrow_mut();

        let metric = metrics.entry(self.name).or_insert(
            Metric {
                time: Duration::new(0, 0),
                count: 0,
                ticker: 0
            }
        );
        metric.time += self.started.elapsed();
        metric.count += 1;
        metric.ticker += self.ticker;

        self.is_stopped = true;
    }

    pub fn set_ticker(&mut self, ticker: usize) {
        self.ticker = ticker;
    }
}

impl<'a> Drop for RunningMetric<'a> {

    // This is where we add the result of this running metric to our hashmap
    fn drop(&mut self) {
        self.stop();
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::thread;

    use std::time;
    #[test]
    fn test_metric() {
        let mut m = Metrics::new();

        {
            let q = m.start("test");
            thread::sleep(time::Duration::from_millis(5));
        }
        assert!(m.metrics.borrow().get("test").unwrap().count == 1);
        assert!(m.metrics.borrow().get("test").unwrap().time.subsec_nanos() >= 5_000_000);
    }
}
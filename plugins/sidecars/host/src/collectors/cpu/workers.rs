/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CPU Load Workers: Each Runs the Kernel for Its Duty Share of Every Worker Slice

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use super::kernel::Kernel;
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Workers {
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
}

struct Shared {
    state: Mutex<State>,
    wake: Condvar,
    operations: AtomicU64,
}

struct State {
    stopped: bool,
    duty: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Workers {
    pub fn start(count: usize, slice: Duration, batch_iterations: u32) -> Result<Self> {
        let mut workers = Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State { stopped: false, duty: 0.0 }),
                wake: Condvar::new(),
                operations: AtomicU64::new(0),
            }),
            threads: Vec::new(),
        };
        for index in 0..count {
            let shared = workers.shared.clone();
            let thread = std::thread::Builder::new()
                .name(format!("host-cpu-load-{index}"))
                .spawn(move || shared.work(slice, batch_iterations))
                .map_err(|error| Error::Invalid(format!("Start CPU Load Workers: {error}; Request Fewer Threads")))?;
            workers.threads.push(thread);
        }
        Ok(workers)
    }

    pub fn duty(&self, duty: f64) {
        self.shared.state.lock().unwrap_or_else(PoisonError::into_inner).duty = duty.clamp(0.0, 1.0);
    }

    pub fn operations(&self) -> u64 {
        self.shared.operations.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.shared.state.lock().unwrap_or_else(PoisonError::into_inner).stopped = true;
        self.shared.wake.notify_all();
        let mut panicked = false;
        for worker in self.threads.drain(..) {
            panicked |= worker.join().is_err();
        }
        if panicked {
            return Err(Error::Invalid("Stop CPU Load Workers: a Worker Panicked; Restart the Plugin".into()));
        }
        Ok(())
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Shared {
    fn work(&self, slice: Duration, batch_iterations: u32) {
        let mut kernel = Kernel::new(batch_iterations);
        loop {
            let start = Instant::now();
            let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.stopped {
                break;
            }
            let busy_until = start + slice.mul_f64(state.duty);
            drop(state);
            while Instant::now() < busy_until {
                if self.state.lock().unwrap_or_else(PoisonError::into_inner).stopped {
                    return;
                }
                self.operations.fetch_add(kernel.batch(), Ordering::Relaxed);
            }
            let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let (state, _) = self
                .wake
                .wait_timeout_while(state, (start + slice).saturating_duration_since(Instant::now()), |state| !state.stopped)
                .unwrap_or_else(PoisonError::into_inner);
            if state.stopped {
                break;
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Workers;

    #[test]
    fn sleeping_workers_stop_without_work() {
        let mut workers = Workers::start(2, Duration::from_secs(1), 4_096).unwrap();
        workers.stop().unwrap();
        assert_eq!(workers.operations(), 0);
    }

    #[test]
    fn active_workers_count_whole_batches() {
        let mut workers = Workers::start(1, Duration::from_millis(10), 4_096).unwrap();
        workers.duty(1.0);
        while workers.operations() == 0 {
            std::thread::yield_now();
        }
        workers.stop().unwrap();
        let operations = workers.operations();
        let batch = super::Kernel::new(4_096).batch();
        assert_eq!(operations % batch, 0);
        assert_eq!(workers.operations(), operations);
    }
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::Instant,
};

// Third Party Imports

// Local Imports
use super::super::kernel::Kernel;
use super::CpuLoadSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Workers {
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
    pub(super) fn start(count: usize, settings: &CpuLoadSettings) -> crate::Result<Self> {
        let mut workers = Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State { stopped: false, duty: 0.0 }),
                wake: Condvar::new(),
                operations: AtomicU64::new(0),
            }),
            threads: Vec::new(),
        };
        let slice = settings.slice();
        let batch_iterations = settings.batch_iterations;
        for index in 0..count {
            let shared = workers.shared.clone();
            let thread = std::thread::Builder::new()
                .name(format!("host-cpu-load-{index}"))
                .spawn(move || {
                    let mut kernel = Kernel::new(batch_iterations);
                    loop {
                        let start = Instant::now();
                        let state = shared.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        if state.stopped {
                            break;
                        }
                        let duty = state.duty;
                        drop(state);
                        let busy_until = start + slice.mul_f64(duty);
                        while Instant::now() < busy_until {
                            if shared.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).stopped {
                                return;
                            }
                            let operations = kernel.batch();
                            shared.operations.fetch_add(operations, Ordering::Relaxed);
                        }
                        let state = shared.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        let (state, _) = shared
                            .wake
                            .wait_timeout_while(state, (start + slice).saturating_duration_since(Instant::now()), |state| !state.stopped)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if state.stopped {
                            break;
                        }
                    }
                })
                .map_err(|error| crate::Error::Probe(error.to_string()))?;
            workers.threads.push(thread);
        }
        Ok(workers)
    }
    pub(super) fn duty(&self, duty: f64) {
        self.shared.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).duty = duty.clamp(0.0, 1.0);
    }
    pub(super) fn operations(&self) -> u64 {
        self.shared.operations.load(Ordering::Relaxed)
    }
    pub(super) fn stop(&mut self) -> crate::Result<()> {
        self.shared.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).stopped = true;
        self.shared.wake.notify_all();
        let mut panicked = false;
        for worker in self.threads.drain(..) {
            panicked |= worker.join().is_err();
        }
        if panicked {
            return Err(crate::Error::Probe("CPU load worker panicked".into()));
        }
        Ok(())
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::CPU_LOAD_DEFAULTS as DEFAULT;
    #[test]
    fn sleeping_workers_wake_on_stop() {
        let settings = CpuLoadSettings {
            slice_ms: 1000,
            period_ms: 1000,
            ..DEFAULT
        };
        let mut workers = Workers::start(2, &settings).unwrap();
        let start = Instant::now();
        workers.stop().unwrap();
        assert!(start.elapsed() < settings.settlement());
        assert_eq!(workers.operations(), 0);
    }
    #[test]
    fn active_workers_stop_and_count_whole_batches() {
        let mut workers = Workers::start(1, &DEFAULT).unwrap();
        workers.duty(1.0);
        let deadline = Instant::now() + DEFAULT.settlement();
        while workers.operations() == 0 && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(workers.operations() > 0);
        let start = Instant::now();
        workers.stop().unwrap();
        assert!(start.elapsed() < DEFAULT.settlement());
        let operations = workers.operations();
        let batch_operations = Kernel::new(DEFAULT.batch_iterations).batch();
        assert_eq!(operations % batch_operations, 0);
        assert_eq!(workers.operations(), operations);
    }
}

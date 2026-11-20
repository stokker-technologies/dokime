/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One Thread Sampling CPU Utilization; Consumers Read the Latest Completed Window

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Condvar, Mutex, PoisonError, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// Third Party Imports
use sysinfo::{CpuRefreshKind, System};

// Local Imports
use super::Load;
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Shared Read Handle on the Latest CPU Load.
#[derive(Clone)]
pub struct Source {
    latest: Arc<RwLock<Load>>,
}

/// Owns the Sampling Thread; Dropping It Stops and Joins the Thread.
pub struct Sampler {
    source: Source,
    stop: Arc<(Mutex<bool>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Source {
    /// A Source Holding One Fixed Load, for Deterministic Controller Tests and Benches.
    pub fn fixed(load: Load) -> Self {
        Self {
            latest: Arc::new(RwLock::new(load)),
        }
    }

    pub fn read(&self) -> Load {
        self.latest.read().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn update(&self, load: Load) {
        *self.latest.write().unwrap_or_else(PoisonError::into_inner) = load;
    }
}

impl Sampler {
    pub fn start(capacity_gflops: f64, period: Duration) -> Result<Self> {
        if period < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            return Err(Error::Invalid(format!(
                "Sample CPU Load: Configure a CPU Sampling Period of at Least {} ms",
                sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis()
            )));
        }
        let source = Source::fixed(Load::measured(capacity_gflops, 0.0));
        let target = source.clone();
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let stopping = stop.clone();
        // Only This Thread Refreshes CPU Deltas.
        let mut system = System::new().map_err(|error| Error::Invalid(format!("Sample CPU Load: {error}; Restart the Plugin")))?;
        system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
        let worker = std::thread::Builder::new()
            .name("host-cpu-sampler".into())
            .spawn(move || {
                let (lock, wake) = &*stopping;
                let mut deadline = Instant::now() + period;
                loop {
                    let stopped = lock.lock().unwrap_or_else(PoisonError::into_inner);
                    let (stopped, _) = wake
                        .wait_timeout_while(stopped, deadline.saturating_duration_since(Instant::now()), |stop| !*stop)
                        .unwrap_or_else(PoisonError::into_inner);
                    if *stopped {
                        break;
                    }
                    drop(stopped);
                    system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
                    let utilization = f64::from(system.global_cpu_usage()).clamp(0.0, 100.0);
                    target.update(Load::measured(capacity_gflops, utilization));
                    deadline += period;
                    if deadline <= Instant::now() {
                        deadline = Instant::now() + period;
                    }
                }
            })
            .map_err(|error| Error::Invalid(format!("Sample CPU Load: {error}; Restart the Plugin")))?;
        Ok(Self {
            source,
            stop,
            worker: Some(worker),
        })
    }

    pub fn source(&self) -> Source {
        self.source.clone()
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        let (lock, wake) = &*self.stop;
        *lock.lock().unwrap_or_else(PoisonError::into_inner) = true;
        wake.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

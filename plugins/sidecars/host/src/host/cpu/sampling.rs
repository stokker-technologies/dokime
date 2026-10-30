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
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// Third Party Imports
use sysinfo::{CpuRefreshKind, System};

// Local Imports
use super::Load;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Source {
    latest: Arc<RwLock<Load>>,
}

pub(crate) struct Sampler {
    source: Source,
    stop: Arc<(Mutex<bool>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Source {
    pub(crate) fn read(&self) -> crate::Result<Load> {
        self.latest.read().map(|sample| sample.clone()).map_err(|_| crate::Error::Probe("CPU sample lock poisoned".into()))
    }
}
impl Sampler {
    pub(crate) fn start(capacity_gflops: f64, period: Duration) -> crate::Result<Self> {
        if period < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            return Err(crate::Error::Probe(format!("CPU sampling period must be at least {:?}", sysinfo::MINIMUM_CPU_UPDATE_INTERVAL)));
        }
        let source = Source {
            latest: Arc::new(RwLock::new(Load::measured(capacity_gflops, 0.0))),
        };
        let target = source.clone();
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let stopping = stop.clone();
        // Only This Thread Refreshes CPU Deltas. Consumers Read A Completed Window.
        let mut system = System::new().map_err(|error| crate::Error::Probe(error.to_string()))?;
        system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
        let worker = std::thread::Builder::new()
            .name("host-cpu-sampler".into())
            .spawn(move || {
                let (lock, wake) = &*stopping;
                let mut deadline = Instant::now() + period;
                loop {
                    let stopped = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    let (stopped, _) = wake
                        .wait_timeout_while(stopped, deadline.saturating_duration_since(Instant::now()), |stop| !*stop)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if *stopped {
                        break;
                    }
                    drop(stopped);
                    system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
                    let utilization = f64::from(system.global_cpu_usage()).clamp(0.0, 100.0);
                    *target.latest.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                        Load::measured(capacity_gflops, utilization);
                    deadline += period;
                    if deadline <= Instant::now() {
                        deadline = Instant::now() + period;
                    }
                }
            })
            .map_err(|error| crate::Error::Probe(error.to_string()))?;
        Ok(Self { source, stop, worker: Some(worker) })
    }
    pub(crate) fn source(&self) -> Source {
        self.source.clone()
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        let (lock, wake) = &*self.stop;
        *lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        wake.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn update(sampler: &Sampler, load: Load) {
        *sampler.source.latest.write().unwrap() = load;
    }
    pub(crate) fn fixed(load: Load) -> Sampler {
        Sampler {
            source: Source { latest: Arc::new(RwLock::new(load)) },
            stop: Arc::new((Mutex::new(false), Condvar::new())),
            worker: None,
        }
    }
}

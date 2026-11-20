/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CPU Collector: Calibrated Capacity, On-Demand Utilization and the Load Control Hand-Off
//
// Calibration runs eight independent f64 FMA accumulators on every logical CPU. Aggregate
// completed work divided by wall time is the calibrated capacity; load is capacity times OS
// utilization. This is a kernel-equivalent estimate, not a hardware peak or instruction
// counter, and each build calibrates its own compiled kernel on the current machine.

/* === Modules === */

pub mod controller;
pub mod hold;
pub mod kernel;
pub mod sampling;
pub mod workers;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use sysinfo::{CpuRefreshKind, System};
use tokio::sync::{OnceCell, broadcast};

// Local Imports
use crate::settings;
use dokime::operations::Context;
use dokime::{Error, Result};
use hold::LoadControl;
use kernel::Kernel;

/* === Constants === */

/// Control Rows Buffered for a Slow Stream Publication Before the Oldest Are Skipped.
pub const CONTROL_ROWS: usize = 4_096;

const OPERATIONS_PER_GIGAFLOP: f64 = 1_000_000_000.0;
const PERCENT_SCALE: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// CPU State Shared by the System Stream, the Load Operations and the Load Control Stream.
///
/// Nothing samples in the background: utilization is read when a row or operation asks for it,
/// and a hold samples only while it runs.
pub struct Collector {
    capacity_gflops: OnceCell<f64>,
    control: broadcast::Sender<LoadControl>,
}

/// Calibrated Capacity, Utilization and Their Product.
#[derive(Clone, Debug, PartialEq, dokime::Message)]
#[dokime(package = "host")]
pub struct Load {
    pub capacity_gflops: f64,
    pub utilization_percent: f64,
    pub load_gflops: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for Collector {
    fn default() -> Self {
        Self {
            capacity_gflops: OnceCell::new(),
            control: broadcast::channel(CONTROL_ROWS).0,
        }
    }
}

impl Collector {
    /// The Calibrated Capacity, Measured Once on First Use.
    pub async fn capacity_gflops(&self, ctx: &Context) -> Result<f64> {
        let capacity_gflops = self
            .capacity_gflops
            .get_or_try_init(|| async {
                let window = settings::duration_value(ctx, settings::CPU_CALIBRATION_WINDOW).await?;
                let batch_iterations = u32::try_from(settings::integer(ctx, settings::CALIBRATION_BATCH_ITERATIONS).await?)
                    .map_err(|_| Error::Invalid("Calibrate CPU Capacity: Configure a Smaller Calibration Batch".into()))?;
                tokio::task::spawn_blocking(move || calibrate(window, batch_iterations))
                    .await
                    .map_err(|error| Error::Invalid(format!("Calibrate CPU Capacity: {error}; Restart the Plugin")))?
            })
            .await?;
        Ok(*capacity_gflops)
    }

    /// Measure Utilization Over sysinfo's Minimum Update Interval and Combine It With the Capacity.
    pub async fn load(&self, ctx: &Context) -> Result<Load> {
        let capacity_gflops = self.capacity_gflops(ctx).await?;
        let utilization_percent = tokio::task::spawn_blocking(|| {
            let mut system = System::new().map_err(|error| Error::Invalid(format!("Read CPU Load: {error}; Restart the Plugin")))?;
            system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
            Ok::<_, Error>(f64::from(system.global_cpu_usage()).clamp(0.0, PERCENT_SCALE))
        })
        .await
        .map_err(|error| Error::Invalid(format!("Read CPU Load: {error}; Restart the Plugin")))??;
        Ok(Load::measured(capacity_gflops, utilization_percent))
    }

    /// Hand One Controller Period's Row to Every Load Control Publication.
    pub fn publish(&self, row: LoadControl) {
        // No Publication Running Means No Demand for the Row.
        let _ = self.control.send(row);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LoadControl> {
        self.control.subscribe()
    }
}

impl Load {
    pub fn measured(capacity_gflops: f64, utilization_percent: f64) -> Self {
        Self {
            capacity_gflops,
            utilization_percent,
            load_gflops: capacity_gflops * utilization_percent / PERCENT_SCALE,
        }
    }
}

/* === Public Functions === */

/// Run the Kernel on Every Logical CPU for the Window and Answer the Capacity in GFLOP/s.
pub fn calibrate(window: Duration, batch_iterations: u32) -> Result<f64> {
    if window.is_zero() || batch_iterations == 0 {
        return Err(Error::Invalid("Calibrate CPU Capacity: Configure a Positive Window and Batch".into()));
    }
    let threads = std::thread::available_parallelism()
        .map_err(|error| Error::Invalid(format!("Calibrate CPU Capacity: {error}; Restart the Plugin")))?
        .get();
    let start = Instant::now();
    let deadline = start + window;
    let operations = std::thread::scope(|scope| {
        let workers = (0..threads).map(|_| scope.spawn(move || Kernel::new(batch_iterations).until(deadline))).collect::<Vec<_>>();
        workers.into_iter().try_fold(0u64, |total, worker| {
            let operations = worker.join().map_err(|_| Error::Invalid("Calibrate CPU Capacity: a Worker Panicked; Restart the Plugin".into()))?;
            total.checked_add(operations).ok_or_else(|| Error::Invalid("Calibrate CPU Capacity: Configure a Shorter Window".into()))
        })
    })?;
    if operations == 0 {
        return Err(Error::Invalid("Calibrate CPU Capacity: No Work Completed; Configure a Longer Calibration Window".into()));
    }
    Ok(operations as f64 / start.elapsed().as_secs_f64() / OPERATIONS_PER_GIGAFLOP)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::Load;

    #[test]
    fn load_is_capacity_times_utilization() {
        let load = Load::measured(12.0, 37.5);
        assert_eq!(load.load_gflops, 4.5);
    }
}

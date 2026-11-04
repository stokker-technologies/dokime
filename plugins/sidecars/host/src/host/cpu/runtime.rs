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
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use super::{CpuLoad, Load};

/* === Constants === */

const OPERATIONS_PER_GIGAFLOP: f64 = 1_000_000_000.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Operation for CpuLoad {
    type Error = dokime::prelude::Refusal;
    async fn run(self, ctx: dokime::harmos::Context<'_, Self>) -> Result<dokime::harmos::Response<Load>, Self::Error> {
        let state = ctx.resource::<tokio::sync::Mutex<super::super::system::State>>();
        let load = dokime::plugin::refusal(state.lock().await.collector.cpu_load())?;
        Ok(dokime::harmos::Response::new(load))
    }
}

/* === Public Functions === */

pub(crate) fn calibrate(budget: Duration, batch_iterations: u32) -> crate::Result<f64> {
    if budget.is_zero() || batch_iterations == 0 {
        return Err(crate::Error::Probe("CPU calibration requires a nonzero operation budget".into()));
    }
    let threads = std::thread::available_parallelism().map_err(|error| crate::Error::Probe(error.to_string()))?.get();
    let start = Instant::now();
    let deadline = start + budget;
    let operations = std::thread::scope(|scope| {
        let workers =
            (0..threads).map(|_| scope.spawn(move || super::kernel::Kernel::new(batch_iterations).until(deadline))).collect::<Vec<_>>();
        workers.into_iter().try_fold(0u64, |total, worker| {
            let operations = worker.join().map_err(|_| crate::Error::Probe("CPU calibration worker panicked".into()))?;
            total.checked_add(operations).ok_or_else(|| crate::Error::Probe("CPU calibration work count overflowed".into()))
        })
    })?;
    if operations == 0 {
        return Err(crate::Error::Probe("CPU calibration completed no work within its operation budget".into()));
    }
    Ok(operations as f64 / start.elapsed().as_secs_f64() / OPERATIONS_PER_GIGAFLOP)
}

/* === Private Functions === */

/* === Tests === */

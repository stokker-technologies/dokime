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
    hint::black_box,
    time::{Duration, Instant},
};

// Third Party Imports

// Local Imports
use super::{CpuLoad, Load};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::Operation for CpuLoad {
    type Error = dokime::prelude::Refusal;
    async fn run(self, ctx: dokime::harmos::Context<'_, Self>) -> Result<dokime::harmos::Response<Load>, Self::Error> {
        let state = ctx.resource::<tokio::sync::Mutex<super::super::system::State>>().clone();
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(move || state.blocking_lock().collector.cpu_load());
        let load = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        Ok(dokime::harmos::Response::new(load))
    }
}

/* === Public Functions === */

pub(crate) fn calibrate(budget: Duration) -> crate::Result<f64> {
    if budget.is_zero() {
        return Err(crate::Error::Probe("CPU calibration requires a nonzero operation budget".into()));
    }
    let threads = std::thread::available_parallelism().map_err(|error| crate::Error::Probe(error.to_string()))?.get();
    let start = Instant::now();
    let deadline = start + budget;
    let operations = std::thread::scope(|scope| {
        let workers = (0..threads).map(|_| scope.spawn(move || kernel(deadline))).collect::<Vec<_>>();
        workers.into_iter().try_fold(0u64, |total, worker| {
            let operations = worker.join().map_err(|_| crate::Error::Probe("CPU calibration worker panicked".into()))?;
            total.checked_add(operations).ok_or_else(|| crate::Error::Probe("CPU calibration work count overflowed".into()))
        })
    })?;
    if operations == 0 {
        return Err(crate::Error::Probe("CPU calibration completed no work within its operation budget".into()));
    }
    Ok(operations as f64 / start.elapsed().as_secs_f64() / 1_000_000_000.0)
}

/* === Private Functions === */

fn kernel(deadline: Instant) -> u64 {
    let mut values = black_box([1.0f64; 8]);
    let multiplier = black_box(1.000_000_000_000_001f64);
    let addend = black_box(0.000_001f64);
    let mut operations = 0;
    while Instant::now() < deadline {
        for _ in 0..4096 {
            for value in &mut values {
                *value = value.mul_add(multiplier, addend);
            }
        }
        black_box(values);
        operations += 4096 * 8 * 2;
    }
    black_box(values);
    operations
}

/* === Tests === */

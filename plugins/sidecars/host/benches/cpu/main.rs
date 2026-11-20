/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CPU Calibration and Load Worker Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use criterion::{BatchSize, Criterion};

// Local Imports
use plugin_host::collectors::cpu::calibrate;
use plugin_host::collectors::cpu::workers::Workers;

/* === Constants === */

const BATCH_ITERATIONS: u32 = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    let mut group = criterion.benchmark_group("cpu");
    group.sample_size(10);

    // Sleeping Workers Must Wake and Stop Well Within One Long Slice.
    group.bench_function("workers/sleeping_settlement", |bench| {
        bench.iter_batched(
            || Workers::start(2, Duration::from_secs(1), BATCH_ITERATIONS).unwrap(),
            |mut workers| workers.stop().unwrap(),
            BatchSize::PerIteration,
        );
    });

    // Busy Workers Stop at Their Next Batch Boundary.
    group.bench_function("workers/active_settlement", |bench| {
        bench.iter_batched(
            || {
                let workers = Workers::start(1, Duration::from_millis(10), BATCH_ITERATIONS).unwrap();
                workers.duty(1.0);
                while workers.operations() == 0 {
                    std::thread::yield_now();
                }
                workers
            },
            |mut workers| workers.stop().unwrap(),
            BatchSize::PerIteration,
        );
    });

    // Capacity Measured on Every Logical CPU Over a Short Window.
    group.bench_function("calibration/100ms", |bench| {
        bench.iter(|| calibrate(Duration::from_millis(100), BATCH_ITERATIONS).unwrap());
    });

    group.finish();
    criterion.final_summary();
}

/* === Tests === */

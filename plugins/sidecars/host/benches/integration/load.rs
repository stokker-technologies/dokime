/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[path = "../../src/host/cpu/load/workers.rs"]
mod workers;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use plugin_host::settings::CpuLoadSettings;

use criterion::{BatchSize, Criterion};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn benchmark(criterion: &mut Criterion) {
    let defaults = dokime::plugin::Configuration::declared::<plugin_host::settings::Settings>()
        .settings::<plugin_host::settings::Settings>()
        .unwrap()
        .cpu_load;
    criterion.bench_function("CPU Workers / Sleeping Settlement", |bench| {
        bench.iter_batched(
            || {
                workers::Workers::start(
                    2,
                    &CpuLoadSettings {
                        slice_ms: 1000,
                        period_ms: 1000,
                        ..defaults.clone()
                    },
                )
                .unwrap()
            },
            |mut workers| workers.stop().unwrap(),
            BatchSize::PerIteration,
        );
    });
    criterion.bench_function("CPU Workers / Active Settlement", |bench| {
        bench.iter_batched(
            || {
                let workers = workers::Workers::start(1, &defaults).unwrap();
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
}

/* === Private Functions === */

/* === Tests === */

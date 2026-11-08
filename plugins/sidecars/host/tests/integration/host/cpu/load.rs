/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[path = "../../../../src/host/cpu/load/workers.rs"]
mod workers;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use plugin_host::settings::CpuLoadSettings;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn sleeping_workers_wake_on_stop() {
    let defaults = dokime::plugin::Configuration::declared::<plugin_host::settings::Settings>()
        .settings::<plugin_host::settings::Settings>()
        .unwrap()
        .cpu_load;
    let settings = CpuLoadSettings {
        slice_ms: 1000,
        period_ms: 1000,
        ..defaults.clone()
    };
    let mut workers = workers::Workers::start(2, &settings).unwrap();
    workers.stop().unwrap();
    assert_eq!(workers.operations(), 0);
}

#[test]
fn active_workers_stop_and_count_whole_batches() {
    let defaults = dokime::plugin::Configuration::declared::<plugin_host::settings::Settings>()
        .settings::<plugin_host::settings::Settings>()
        .unwrap()
        .cpu_load;
    let mut workers = workers::Workers::start(1, &defaults).unwrap();
    workers.duty(1.0);
    while workers.operations() == 0 {
        std::thread::yield_now();
    }
    assert!(workers.operations() > 0);
    workers.stop().unwrap();
    let operations = workers.operations();
    let batch_operations = super::kernel::Kernel::new(defaults.batch_iterations).batch();
    assert_eq!(operations % batch_operations, 0);
    assert_eq!(workers.operations(), operations);
}

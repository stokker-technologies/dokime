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

// Third Party Imports

// Local Imports
use dokime::harmos::{
    Message,
    operations::{OperationDeclaration, Visibility},
};
use plugin_host::{
    collectors::system::SystemCollector,
    host::{
        cpu::{CpuLoad, Load},
        system::POLICY,
    },
};

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

#[dokime_tracing::trace_test]
#[test]
fn calibrated_cpu() {
    let mut collector = SystemCollector::new(plugin_host::HostOptions::default()).unwrap();
    collector.calibrate_cpu(POLICY.attempt_timeout / 2).unwrap();
    let load = collector.cpu_load().unwrap();
    assert!(load.capacity_gflops.is_finite() && load.capacity_gflops > 0.0);
    assert!((0.0..=100.0).contains(&load.utilization_percent));
    assert_eq!(load.load_gflops, load.capacity_gflops * load.utilization_percent / 100.0);
    assert_eq!(CpuLoad::VISIBILITY, Visibility::Public);
    assert!(load == Load::decode(&load.encode()).unwrap());
    let row = collector.collect().unwrap();
    assert!(row.values.contains_key("cpu_capacity_gflops"));
    assert!(row.values.contains_key("cpu_load_gflops"));
}

#[cfg(target_os = "windows")]
#[dokime_tracing::trace_test]
#[test]
fn active_power_plan() {
    use plugin_host::host::power::{GetPlan, Plan, active_plan};
    let plan = active_plan().unwrap();
    assert_eq!(GetPlan::VISIBILITY, Visibility::Public);
    assert!(!plan.id.is_empty());
    assert!(plan == Plan::decode(&plan.encode()).unwrap());
}

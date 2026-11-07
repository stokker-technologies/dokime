/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Resource Isolation and Cleanup

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

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

#[dokime_tracing::trace_test]
#[test]
fn operation_resources_isolate_devices_and_release_owned_logs() {
    use plugin_obc::domain::model::charger::{Fault, Power, Supply};
    use plugin_obc::operations::charger::{Chargers, SampleChargerBatch};
    let devices = Chargers::default();
    let supply = Supply {
        voltage_rms_v: 230.0,
        load_ohm: 100.0,
        frequency_hz: 50.0,
        phase_deg: 15.0,
        current_limit_a: 5.0,
    };
    let first = devices.initialize(supply, true).unwrap();
    let second = devices.initialize(Supply { voltage_rms_v: 120.0, ..supply }, false).unwrap();
    assert_ne!(first.device_id, second.device_id);
    devices.start(first.device_id).unwrap();
    devices.start(second.device_id).unwrap();
    devices.inject_fault(first.device_id, Fault::PowerLoss).unwrap();
    assert!(devices.probe_restart(first.device_id).unwrap().blocked);
    let operation = SampleChargerBatch {
        device_id: second.device_id,
        first_index: 3,
        count: 4,
        interval_s: 0.01,
    };
    let samples = operation.sample(&devices).unwrap();
    assert_eq!(samples.rows.len(), 4);
    assert!(samples.rows.iter().all(|sample| sample.reading.current_rms_a > 0.0 && sample.charger.power == Power::Running));
    assert_eq!(devices.read(first.device_id).unwrap().fault, Some(Fault::PowerLoss));
    let log = std::path::PathBuf::from(first.log.unwrap());
    devices.sample(first.device_id, 0.1).unwrap();
    assert!(std::fs::metadata(&log).unwrap().len() > 0);
    devices.release(first.device_id).unwrap();
    assert!(!log.exists());
    assert!(devices.read(first.device_id).is_err());
    assert_eq!(devices.read(second.device_id).unwrap().power, Power::Running);
    devices.release(second.device_id).unwrap();
}

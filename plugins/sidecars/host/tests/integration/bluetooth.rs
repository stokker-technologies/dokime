/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Telemetry Stories: the Deep Refresh Cadence and Its Cached Inventory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use plugin_host::collectors::bluetooth::State;
use plugin_host::domain::readings::Bluetooth;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn inventory(adapters: i64) -> Bluetooth {
    Bluetooth {
        bluetooth_adapter_count: Some(adapters),
        bluetooth_device_names: vec!["Headset".into()],
        ..Bluetooth::default()
    }
}

/* === Tests === */

#[test]
fn the_deep_refresh_interval_controls_native_inventories() -> dokime::Result<()> {
    let mut state = State::default();
    let opened = Instant::now();
    let interval = Duration::from_millis(250);
    let mut probes = Vec::new();
    for (elapsed, expected) in [(0, 1), (249, 1), (250, 2), (499, 2), (500, 3)] {
        let row = state.collect_at(opened + Duration::from_millis(elapsed), interval, || {
            probes.push(elapsed);
            Ok(inventory(i64::try_from(probes.len()).unwrap()))
        })?;
        assert_eq!(row.bluetooth_adapter_count, Some(expected));
        assert_eq!(row.probe_status.as_deref(), Some("ok"));
        assert_eq!(row.bluetooth_device_names, vec!["Headset".to_owned()]);
    }
    assert_eq!(probes, vec![0, 250, 500]);
    Ok(())
}

#[test]
fn a_failed_refresh_reuses_the_cached_inventory_as_stale() -> dokime::Result<()> {
    let mut state = State::default();
    let opened = Instant::now();
    let interval = Duration::from_millis(100);
    let failed = state.collect_at(opened, interval, || Err("unavailable".into()))?;
    assert_eq!(failed.probe_status.as_deref(), Some("error"));
    assert_eq!(failed.probe_error.as_deref(), Some("unavailable"));
    assert_eq!(failed.bluetooth_adapter_count, None);
    let between = state.collect_at(opened + Duration::from_millis(50), interval, || Ok(inventory(9)))?;
    assert_eq!(between.probe_status.as_deref(), Some("unavailable"));
    let fresh = state.collect_at(opened + interval, interval, || Ok(inventory(1)))?;
    assert_eq!(fresh.probe_status.as_deref(), Some("ok"));
    let stale = state.collect_at(opened + interval * 2, interval, || Err("gone".into()))?;
    assert_eq!(stale.probe_status.as_deref(), Some("stale"));
    assert_eq!(stale.probe_error.as_deref(), Some("gone"));
    assert_eq!(stale.bluetooth_adapter_count, Some(1));
    assert_eq!(stale.sample_index, Some(4));
    Ok(())
}

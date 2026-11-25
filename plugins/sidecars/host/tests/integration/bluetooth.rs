/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Telemetry Stories: the Inventory's Deep Refresh Cadence, the Link's Radio Throughput and Advertised RSSI

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use plugin_host::collectors::bluetooth::{Heard, Measure, Radio, Sample, State, link_row, radios, rate, select};
use plugin_host::domain::readings::Bluetooth;
use plugin_host::domain::telemetry::BluetoothRssi;
use plugin_host::settings;

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

fn sample(instance: &str, measure: Measure, value: f64) -> Sample {
    Sample {
        instance: instance.into(),
        measure,
        value,
    }
}

fn rssi(name: &str, dbm: f64) -> BluetoothRssi {
    BluetoothRssi {
        bluetooth_device_name: name.into(),
        bluetooth_rssi_dbm: dbm,
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

#[test]
fn link_throughput_sums_every_radio_counter_into_megabits() {
    // Classic ACL, LE ACL and SCO Bytes Add Up per Direction; Classic and LE Connections Add Up per Radio.
    let samples = [
        sample("Radio A", Measure::Received, 100_000.0),
        sample("Radio A", Measure::Received, 20_000.0),
        sample("Radio A", Measure::Received, 5_000.0),
        sample("Radio A", Measure::Transmitted, 50_000.0),
        sample("Radio A", Measure::Connections, 2.0),
        sample("Radio A", Measure::Connections, 1.0),
        sample("Radio B", Measure::Received, 125_000.0),
        sample("Radio B", Measure::Transmitted, 0.0),
        sample("Radio B", Measure::Connections, 1.0),
    ];
    let radios = radios(samples);
    assert_eq!(radios.len(), 2);
    assert_eq!(radios[0].received, Some(125_000.0));
    assert_eq!(radios[0].connections, Some(3));
    let row = link_row(Ok(radios), Ok(Vec::new()), None);
    assert_eq!(row.probe_status.as_deref(), Some("ok"));
    assert_eq!(row.probe_error, None);
    assert_eq!(row.bluetooth_rx_mbps, Some(2.0));
    assert_eq!(row.bluetooth_tx_mbps, Some(0.4));
    assert_eq!(row.bluetooth_connected_device_count, Some(4));
}

#[test]
fn the_adapter_setting_selects_radios_by_name_and_names_the_present_ones() {
    let radio = |name: &str, received: f64| Radio {
        names: vec![name.into(), format!("/org/bluez/{name}")],
        received: Some(received),
        transmitted: Some(0.0),
        connections: Some(1),
    };
    let present = || vec![radio("hci0", 125_000.0), radio("hci1", 250_000.0)];
    let selected = select(present(), Some("HCI1")).unwrap();
    assert_eq!(selected, vec![radio("hci1", 250_000.0)]);
    assert_eq!(select(present(), Some("/org/bluez/hci0")).unwrap(), vec![radio("hci0", 125_000.0)]);
    assert_eq!(select(present(), None).unwrap().len(), 2);
    let row = link_row(Ok(present()), Ok(Vec::new()), Some("hci1"));
    assert_eq!((row.bluetooth_rx_mbps, row.bluetooth_connected_device_count), (Some(2.0), Some(1)));
    let unmatched = link_row(Ok(present()), Ok(Vec::new()), Some("Dongle"));
    assert_eq!(unmatched.probe_status.as_deref(), Some("unavailable"));
    let error = unmatched.probe_error.unwrap();
    assert!(error.contains("hci0, hci1") && error.contains("Bluetooth Adapter"), "{error}");
    assert_eq!((unmatched.bluetooth_rx_mbps, unmatched.bluetooth_connected_device_count), (None, None));
}

#[test]
fn the_first_tick_and_a_missing_radio_publish_no_throughput() {
    // Connection Counts Are Instantaneous, So the First Tick Already Has Them; Rates Need a Second Collection.
    let first = link_row(Ok(radios([sample("Radio", Measure::Connections, 1.0)])), Ok(Vec::new()), None);
    assert_eq!(first.probe_status.as_deref(), Some("pending"));
    assert!(first.probe_error.unwrap().contains("Second Collection"));
    assert_eq!((first.bluetooth_rx_mbps, first.bluetooth_tx_mbps), (None, None));
    assert_eq!(first.bluetooth_connected_device_count, Some(1));
    let absent = link_row(Ok(Vec::new()), Err("Watch Bluetooth Advertisements: No Bluetooth Radio Is Available".into()), None);
    assert_eq!(absent.probe_status.as_deref(), Some("unavailable"));
    let error = absent.probe_error.unwrap();
    assert!(error.contains("No Bluetooth Radio Is Present") && error.contains(" | Watch Bluetooth Advertisements"), "{error}");
    assert_eq!((absent.bluetooth_rx_mbps, absent.bluetooth_connected_device_count), (None, None));
    let failed = link_row(Err("Read Bluetooth Counters: denied".into()), Ok(vec![rssi("Sensor", -60.0)]), None);
    assert_eq!(failed.probe_status.as_deref(), Some("error"));
    assert_eq!(failed.bluetooth_rssi, vec![rssi("Sensor", -60.0)]);
}

#[test]
fn byte_counters_rate_over_the_elapsed_time_across_a_wrap() {
    assert_eq!(rate(100, 1_100, 0.5), 2_000.0);
    assert_eq!(rate(u32::MAX - 99, 900, 1.0), 1_000.0);
    assert_eq!(rate(7, 7, 1.0), 0.0);
}

#[test]
fn each_tick_reports_the_newest_rssi_of_every_named_device_heard_since_the_last() {
    let opened = Instant::now();
    let at = |ms| opened + Duration::from_millis(ms);
    let mut heard = Heard::default();
    heard.record("Sensor", -70.0, at(0));
    heard.record("Sensor", -64.0, at(400));
    heard.record("Sensor", -90.0, at(200));
    heard.record("", -40.0, at(100));
    heard.record("  ", -40.0, at(100));
    heard.record("Phone", -55.0, at(300));
    assert_eq!(heard.drain(&[]), vec![rssi("Phone", -55.0), rssi("Sensor", -64.0)]);
    // A Device Silent During the Next Tick Gets No Row.
    heard.record("Phone", -58.0, at(1_100));
    assert_eq!(heard.drain(&[]), vec![rssi("Phone", -58.0)]);
    assert_eq!(heard.drain(&[]), Vec::new());
}

#[test]
fn the_rssi_devices_setting_selects_names_by_substring_ignoring_case() {
    let devices = settings::split(" sensor , ,Tag 2,");
    assert_eq!(devices, vec!["sensor".to_owned(), "Tag 2".to_owned()]);
    assert!(settings::split("  ").is_empty());
    let opened = Instant::now();
    let mut heard = Heard::default();
    for name in ["Lab Sensor 1", "SENSOR-B", "tag 2", "Tag 3", "Headset"] {
        heard.record(name, -60.0, opened);
    }
    let names = heard.drain(&devices).into_iter().map(|row| row.bluetooth_device_name).collect::<Vec<_>>();
    assert_eq!(names, vec!["Lab Sensor 1".to_owned(), "SENSOR-B".to_owned(), "tag 2".to_owned()]);
}

#[test]
fn one_link_collection_declares_bluetooth_throughput_connections_and_rssi() -> dokime::Result<()> {
    let declarations = plugin_host::declarations()?;
    let streams = declarations.streams.iter().map(|stream| stream.id.as_str()).collect::<Vec<_>>();
    for stream in ["host.bluetooth_link_probe", "host.bluetooth_throughput", "host.bluetooth_connections", "host.bluetooth_rssi"] {
        assert_eq!(streams.iter().filter(|id| **id == stream).count(), 1, "{stream}");
    }
    assert!(!streams.contains(&"host.bluetooth_traffic"));
    Ok(())
}

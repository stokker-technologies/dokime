/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Telemetry Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use tokio::sync::oneshot;

// Local Imports
use plugin_host::collectors::system::BatteryProbe;
use plugin_host::domain::readings::System;

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
fn absent_battery_is_cached_until_deep_refresh_or_settings_change() {
    let mut probe = BatteryProbe::<System>::default();
    let now = Instant::now();
    let interval = Duration::from_secs(300);
    let (sender, receiver) = oneshot::channel();
    assert!(probe.collect_at(now, interval, || receiver).is_none());
    sender.send(None).unwrap();
    assert!(probe.collect_at(now, interval, || panic!("Inventory Is Already Pending")).is_none());
    assert!(probe.collect_at(now + interval - Duration::from_secs(1), interval, || panic!("Absence Is Cached")).is_none());

    let (sender, receiver) = oneshot::channel();
    assert!(probe.collect_at(now + interval, interval, || receiver).is_none());
    sender.send(None).unwrap();
    assert!(probe.collect_at(now + interval, interval, || panic!("Inventory Is Already Pending")).is_none());
    let (sender, receiver) = oneshot::channel();
    assert!(probe.collect_at(now + interval + Duration::from_secs(1), Duration::from_secs(1), || receiver).is_none());
    sender
        .send(Some(System {
            battery_percent: Some(75.0),
            ..System::default()
        }))
        .unwrap();
    let (_sender, receiver) = oneshot::channel();
    let row = probe.collect_at(now + interval + Duration::from_secs(1), interval, || receiver).unwrap();
    assert_eq!(row.battery_percent, Some(75.0));
}

#[test]
fn pending_battery_probe_returns_without_waiting_or_starting_another_probe() {
    let mut probe = BatteryProbe::<System>::default();
    let now = Instant::now();
    let interval = Duration::from_secs(300);
    let (sender, receiver) = oneshot::channel();
    assert!(probe.collect_at(now, interval, || receiver).is_none());
    // The Fake Platform Holds Its Battery Reply Across Several System Ticks.
    for index in 1..=3 {
        assert!(probe.collect_at(now + interval * index, interval, || panic!("Only One Probe May Run")).is_none());
    }
    sender
        .send(Some(System {
            battery_percent: Some(42.0),
            ..System::default()
        }))
        .unwrap();
    let (sender, receiver) = oneshot::channel();
    assert_eq!(probe.collect_at(now + interval * 3, interval, || receiver).unwrap().battery_percent, Some(42.0));
    sender
        .send(Some(System {
            battery_percent: Some(41.0),
            ..System::default()
        }))
        .unwrap();
    let (_sender, receiver) = oneshot::channel();
    assert_eq!(probe.collect_at(now + interval * 4, interval, || receiver).unwrap().battery_percent, Some(41.0));
}

#[test]
fn a_new_activation_discards_the_previous_inventory_and_pending_reply() {
    let mut probe = BatteryProbe::<System>::default();
    let now = Instant::now();
    let interval = Duration::from_secs(300);
    let (sender, receiver) = oneshot::channel();
    probe.collect_at(now, interval, || receiver);
    drop(probe);
    assert!(sender.send(None).is_err());
    let mut probe = BatteryProbe::<System>::default();
    let (sender, receiver) = oneshot::channel();
    probe.collect_at(now, interval, || receiver);
    sender.send(None).unwrap();
    assert!(probe.collect_at(now, interval, || panic!("Read the Completed Probe")).is_none());
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Collector State Shared by Its Observers: the Inventory Caches Present Devices Between Deep
// Refreshes, the Link Reads Radio Counters and Heard Advertisements Every Tick; Neither Reads the Other's State.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use crate::collectors::megabits_per_second;
use crate::controls::Timing;
use crate::domain::readings::{Bluetooth, BluetoothLink};
use crate::domain::telemetry::BluetoothRssi;
use crate::platform::bluetooth::Link;
use dokime::{Error, Result};

/* === Constants === */

const NO_RADIO: &str = "Read Bluetooth Counters: No Bluetooth Radio Is Present; Connect or Turn On a Bluetooth Radio";
const PENDING: &str = "Read Bluetooth Counters: Rates Need a Second Collection; Wait One Bluetooth Link Interval";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Bluetooth Collection State, Each Part Released at Its Collection's Stop.
#[derive(Default)]
pub struct Collector {
    inventory: Arc<Mutex<State>>,
    link: Arc<Mutex<Option<Link>>>,
}

/// What the Inventory Collection Reads From the Plugin Settings.
pub struct InventoryOptions {
    pub adapter: Option<String>,
    pub probe_timeout: Duration,
    pub deep_refresh_interval: Duration,
    pub timing: Timing,
}

/// What the Link Collection Reads From the Plugin Settings.
pub struct LinkOptions {
    pub adapter: Option<String>,
    pub rssi_devices: Vec<String>,
    pub probe_timeout: Duration,
    pub minimum_rate_window: Duration,
}

/// The Sample Count and the Last Native Inventory, Reused Between Deep Refreshes.
#[derive(Default)]
pub struct State {
    sample_index: u64,
    last_deep_refresh: Option<Instant>,
    cached: Option<Bluetooth>,
}

/// One Radio at One Link Collection: Its Names, Traffic in Bytes per Second and Connections; a Rate Is Absent Before Its Second Collection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Radio {
    pub names: Vec<String>,
    pub received: Option<f64>,
    pub transmitted: Option<f64>,
    pub connections: Option<i64>,
}

/// One Counter Value of One Radio Instance.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub instance: String,
    pub measure: Measure,
    pub value: f64,
}

/// Advertisements Heard Since the Previous Drain: the Newest Signal Strength per Advertised Name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Heard(BTreeMap<String, (Instant, f64)>);

/* === Enums === */

/// Which Link Measurement a Counter Adds To.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measure {
    Received,
    Transmitted,
    Connections,
}

/* === Traits === */

/* === Implementations === */

impl Collector {
    /// One Inventory Row; the Native Inventory Runs Only When a Deep Refresh Is Due.
    pub async fn inventory(&self, options: InventoryOptions) -> Result<Bluetooth> {
        let state = Arc::clone(&self.inventory);
        blocking(move || {
            let mut state = state.lock().map_err(|_| poisoned())?;
            state.collect_at(Instant::now(), options.deep_refresh_interval, || inventory(&options))
        })
        .await
    }

    /// Admit Link Ticks When Link Collection Demand Starts, So a Tick Settling After the Stop Opens Nothing.
    pub fn start_link(&self) -> Result<()> {
        *self.link.lock().map_err(|_| poisoned())? = Some(Link::default());
        Ok(())
    }

    /// One Link Row Read Fresh Every Tick; the Counters and the Advertisement Watcher Open at the First Tick.
    pub async fn link(&self, options: LinkOptions) -> Result<BluetoothLink> {
        let state = Arc::clone(&self.link);
        blocking(move || {
            let mut state = state.lock().map_err(|_| poisoned())?;
            let link =
                state.as_mut().ok_or_else(|| Error::Invalid("Collect Bluetooth Link: Start the Collection Before Reading".into()))?;
            let radios = link.radios(&options);
            let heard = link.heard(&options.rssi_devices);
            Ok(link_row(radios, heard, options.adapter.as_deref()))
        })
        .await
    }
}

impl Collector {
    /// Drop the Cached Inventory When the Inventory Collection's Last Listener Leaves.
    pub fn stop_inventory(&self) {
        *self.inventory.lock().unwrap_or_else(PoisonError::into_inner) = State::default();
    }

    /// Close the Counters and Stop the Watcher When the Link Collection's Last Listener Leaves.
    pub fn stop_link(&self) {
        *self.link.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

impl State {
    /// One Row at `now`: a Fresh Inventory When the Deep Refresh Is Due, Else the Cached One; a Failed Refresh Reuses the Cache as Stale.
    pub fn collect_at(
        &mut self, now: Instant, deep_refresh_interval: Duration, probe: impl FnOnce() -> std::result::Result<Bluetooth, String>,
    ) -> Result<Bluetooth> {
        self.sample_index += 1;
        let refresh = self.last_deep_refresh.is_none_or(|last| now.duration_since(last) >= deep_refresh_interval);
        let (mut row, status) = if refresh {
            self.last_deep_refresh = Some(now);
            match probe() {
                Ok(snapshot) => {
                    self.cached = Some(snapshot.clone());
                    (snapshot, "ok")
                }
                Err(error) => {
                    let (mut row, status) = match self.cached.clone() {
                        Some(cached) => (cached, "stale"),
                        None => (Bluetooth::default(), "error"),
                    };
                    row.probe_error = Some(error);
                    (row, status)
                }
            }
        } else {
            match self.cached.clone() {
                Some(cached) => (cached, "ok"),
                None => (Bluetooth::default(), "unavailable"),
            }
        };
        row.sample_index = Some(i64::try_from(self.sample_index).unwrap_or(i64::MAX));
        row.probe_status = Some(status.into());
        Ok(row)
    }
}

impl Heard {
    /// Record One Advertisement; an Unnamed One Is Skipped and an Older One Never Replaces a Newer.
    pub fn record(&mut self, name: &str, rssi_dbm: f64, at: Instant) {
        let name = name.trim();
        if name.is_empty() || self.0.get(name).is_some_and(|(newest, _)| *newest > at) {
            return;
        }
        self.0.insert(name.to_owned(), (at, rssi_dbm));
    }

    /// One Row per Selected Device Heard Since the Previous Drain, With Its Newest Signal Strength; Silent Devices Have No Row.
    pub fn drain(&mut self, devices: &[String]) -> Vec<BluetoothRssi> {
        std::mem::take(&mut self.0)
            .into_iter()
            .filter(|(name, _)| devices.is_empty() || devices.iter().any(|device| matches(name, device)))
            .map(|(bluetooth_device_name, (_, bluetooth_rssi_dbm))| BluetoothRssi {
                bluetooth_device_name,
                bluetooth_rssi_dbm,
            })
            .collect()
    }
}

/* === Public Functions === */

/// Sum Counter Samples per Radio Instance; a Measurement Without a Valid Sample Stays Absent.
pub fn radios(samples: impl IntoIterator<Item = Sample>) -> Vec<Radio> {
    let mut radios = BTreeMap::<String, Radio>::new();
    for Sample { instance, measure, value } in samples {
        let radio = radios.entry(instance.clone()).or_insert_with(|| Radio {
            names: vec![instance],
            ..Radio::default()
        });
        match measure {
            Measure::Received => radio.received = Some(radio.received.unwrap_or_default() + value),
            Measure::Transmitted => radio.transmitted = Some(radio.transmitted.unwrap_or_default() + value),
            Measure::Connections => radio.connections = Some(radio.connections.unwrap_or_default() + value.round() as i64),
        }
    }
    radios.into_values().collect()
}

/// The Radios Whose Names Contain the Configured Adapter, Ignoring Case; Every Radio When None Is Configured.
pub fn select(radios: Vec<Radio>, adapter: Option<&str>) -> std::result::Result<Vec<Radio>, String> {
    if radios.is_empty() {
        return Err(NO_RADIO.into());
    }
    let Some(adapter) = adapter else {
        return Ok(radios);
    };
    let present = radios.iter().filter_map(|radio| radio.names.first().cloned()).collect::<Vec<_>>().join(", ");
    let selected = radios.into_iter().filter(|radio| radio.names.iter().any(|name| matches(name, adapter))).collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(format!(
            "Select Bluetooth Radio: No Radio Matches {adapter:?} Among {present}; Name One in the Bluetooth Adapter Setting"
        ));
    }
    Ok(selected)
}

/// Bytes per Second Between Two Readings of a Wrapping 32-Bit Byte Counter.
pub fn rate(previous: u32, current: u32, elapsed_s: f64) -> f64 {
    f64::from(current.wrapping_sub(previous)) / elapsed_s
}

/// The Link Row of One Tick: the Selected Radios' Throughput and Connections and the Devices Heard; the Outcome Names the First Gap.
pub fn link_row(
    radios: std::result::Result<Vec<Radio>, String>, heard: std::result::Result<Vec<BluetoothRssi>, String>, adapter: Option<&str>,
) -> BluetoothLink {
    let mut row = BluetoothLink::default();
    let mut outcome = None;
    let mut errors = Vec::new();
    let mut gap = |status: &'static str, error: String| {
        outcome.get_or_insert(status);
        errors.push(error);
    };
    match radios {
        Err(error) => gap("error", error),
        Ok(radios) => match select(radios, adapter) {
            Err(error) => gap("unavailable", error),
            Ok(selected) => {
                row.bluetooth_connected_device_count =
                    selected.iter().filter_map(|radio| radio.connections).reduce(|sum, count| sum + count);
                let total = |rate: fn(&Radio) -> Option<f64>| selected.iter().map(rate).sum::<Option<f64>>();
                match (total(|radio| radio.received), total(|radio| radio.transmitted)) {
                    (Some(received), Some(transmitted)) => {
                        row.bluetooth_rx_mbps = Some(megabits_per_second(received));
                        row.bluetooth_tx_mbps = Some(megabits_per_second(transmitted));
                    }
                    _ => gap("pending", PENDING.into()),
                }
            }
        },
    }
    match heard {
        Ok(heard) => row.bluetooth_rssi = heard,
        Err(error) => gap("unavailable", error),
    }
    row.probe_status = Some(outcome.unwrap_or("ok").into());
    row.probe_error = (!errors.is_empty()).then(|| errors.join(" | "));
    row
}

/* === Private Functions === */

async fn blocking<T: Send + 'static>(collect: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(collect).await.map_err(|error| {
        Error::Invalid(format!("Collect Bluetooth Telemetry: the collection worker failed: {error}; Restart the Plugin"))
    })?
}

fn poisoned() -> Error {
    Error::Invalid("Collect Bluetooth Telemetry: a previous collection panicked; Restart the Plugin".into())
}

fn matches(name: &str, filter: &str) -> bool {
    name.to_lowercase().contains(&filter.to_lowercase())
}

#[cfg(windows)]
fn inventory(options: &InventoryOptions) -> std::result::Result<Bluetooth, String> {
    crate::platform::bluetooth::snapshot(options.adapter.as_deref(), &options.timing)
}

#[cfg(unix)]
fn inventory(options: &InventoryOptions) -> std::result::Result<Bluetooth, String> {
    crate::platform::bluetooth::snapshot(options.adapter.as_deref(), options.probe_timeout)
}

/* === Tests === */

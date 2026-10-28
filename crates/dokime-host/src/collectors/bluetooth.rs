/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[cfg(not(windows))]
use std::process::Command;

use crate::{HostOptions, HostStreamSample, HostTelemetryStream, Result, collectors::maybe_insert};
use dokime_domain::model::values::Value;

const BLUETOOTH_DEEP_SCAN_INTERVAL_MS: u64 = 300_000;
#[cfg(not(windows))]
const BLUETOOTH_COMMAND_TIMEOUT_MS: u64 = 5_000;
const WINDOWS_BLUETOOTH_TIMEOUT_MS: u64 = 10_000;

pub struct BluetoothCollector {
    sample_index: u64,
    options: HostOptions,
    deep_refresh_interval: Duration,
    last_deep_refresh: Option<Instant>,
    cached_values: Option<BTreeMap<String, Value>>,
}

impl BluetoothCollector {
    pub fn new(options: HostOptions) -> Self {
        let deep_refresh_interval = Duration::from_millis(BLUETOOTH_DEEP_SCAN_INTERVAL_MS);
        Self {
            sample_index: 0,
            options,
            deep_refresh_interval,
            last_deep_refresh: None,
            cached_values: None,
        }
    }

    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("unsupported")),
        ]);

        #[cfg(windows)]
        {
            let now = Instant::now();
            let should_refresh =
                self.last_deep_refresh.map(|last_refresh| now.duration_since(last_refresh) >= self.deep_refresh_interval).unwrap_or(true);

            if should_refresh {
                self.last_deep_refresh = Some(now);
                match crate::platform::windows::bluetooth::snapshot(&self.options, Duration::from_millis(WINDOWS_BLUETOOTH_TIMEOUT_MS)) {
                    Ok(snapshot) => {
                        let snapshot_values = snapshot_values(snapshot);
                        self.cached_values = Some(snapshot_values.clone());
                        values.insert("probe_status".to_owned(), Value::from("ok"));
                        values.extend(snapshot_values);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        if let Some(cached_values) = &self.cached_values {
                            values.insert("probe_status".to_owned(), Value::from("stale"));
                            values.insert("probe_error".to_owned(), Value::from(message));
                            values.extend(cached_values.clone());
                        } else {
                            values.insert("probe_status".to_owned(), Value::from("error"));
                            values.insert("probe_error".to_owned(), Value::from(message));
                        }
                    }
                }
            } else if let Some(cached_values) = &self.cached_values {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.extend(cached_values.clone());
            }
        }

        #[cfg(not(windows))]
        {
            let now = Instant::now();
            let should_refresh =
                self.last_deep_refresh.map(|last_refresh| now.duration_since(last_refresh) >= self.deep_refresh_interval).unwrap_or(true);

            if should_refresh {
                self.last_deep_refresh = Some(now);
                match crate::platform::linux::bluetooth::linux_bluetooth_snapshot(
                    &self.options,
                    Duration::from_millis(BLUETOOTH_COMMAND_TIMEOUT_MS),
                ) {
                    Ok(snapshot_values) => {
                        self.cached_values = Some(snapshot_values.clone());
                        values.insert("probe_status".to_owned(), Value::from("ok"));
                        values.extend(snapshot_values);
                    }
                    Err(error) => {
                        if let Some(cached_values) = &self.cached_values {
                            values.insert("probe_status".to_owned(), Value::from("stale"));
                            values.insert("probe_error".to_owned(), Value::from(error));
                            values.extend(cached_values.clone());
                        } else {
                            values.insert("probe_status".to_owned(), Value::from("error"));
                            values.insert("probe_error".to_owned(), Value::from(error));
                        }
                    }
                }
            } else if let Some(cached_values) = &self.cached_values {
                values.insert("probe_status".to_owned(), Value::from("ok"));
                values.extend(cached_values.clone());
            }
        }

        Ok(HostStreamSample::new(HostTelemetryStream::Bluetooth, values))
    }
}

#[cfg(windows)]
fn snapshot_values(snapshot: crate::platform::windows::bluetooth::BluetoothSnapshot) -> BTreeMap<String, Value> {
    let mut values = BTreeMap::new();
    maybe_insert(&mut values, "bluetooth_adapter", snapshot.adapter_name.map(Value::from));
    values.insert("bluetooth_adapter_count".to_owned(), Value::Integer(snapshot.adapter_count as i64));
    maybe_insert(
        &mut values,
        "bluetooth_connected_device_count",
        snapshot.connected_device_count.map(|count| Value::Integer(count as i64)),
    );
    values.insert("bluetooth_present_device_count".to_owned(), Value::Integer(snapshot.present_device_count as i64));
    maybe_insert(&mut values, "bluetooth_radio_state", snapshot.radio_state.map(Value::from));
    values.insert("bluetooth_device_names".to_owned(), Value::Array(snapshot.device_names.into_iter().map(Value::from).collect()));

    values
}

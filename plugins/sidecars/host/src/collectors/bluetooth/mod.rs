/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

// Third Party Imports

// Local Imports
use crate::{HostOptions, HostStreamSample, HostTelemetryStream, Result};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct BluetoothCollector {
    sample_index: u64,
    options: HostOptions,
    deep_refresh_interval: Duration,
    last_deep_refresh: Option<Instant>,
    cached_values: Option<BTreeMap<String, Value>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl BluetoothCollector {
    pub fn new(options: HostOptions) -> Self {
        let deep_refresh_interval = crate::config::BLUETOOTH_DEEP_REFRESH;
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
            ("probe_status".to_owned(), Value::from("unavailable")),
            ("bluetooth_device_names".to_owned(), Value::Array(Vec::new())),
        ]);

        let now = Instant::now();
        let refresh = self.last_deep_refresh.map(|last| now.duration_since(last) >= self.deep_refresh_interval).unwrap_or(true);
        if refresh {
            self.last_deep_refresh = Some(now);
            match Self::bluetooth_values(&self.options, crate::host::bluetooth::POLICY.attempt_timeout) {
                Ok(snapshot) => {
                    self.cached_values = Some(snapshot.clone());
                    values.insert("probe_status".to_owned(), Value::from("ok"));
                    values.extend(snapshot);
                }
                Err(error) => {
                    values.insert("probe_error".to_owned(), Value::from(error));
                    if let Some(cached) = &self.cached_values {
                        values.insert("probe_status".to_owned(), Value::from("stale"));
                        values.extend(cached.clone());
                    } else {
                        values.insert("probe_status".to_owned(), Value::from("error"));
                    }
                }
            }
        } else if let Some(cached) = &self.cached_values {
            values.insert("probe_status".to_owned(), Value::from("ok"));
            values.extend(cached.clone());
        }

        Ok(HostStreamSample::new(HostTelemetryStream::Bluetooth, values))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

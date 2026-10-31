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
        let deep_refresh_interval = Duration::from_millis(options.timing.bluetooth_deep_refresh_ms);
        Self {
            sample_index: 0,
            options,
            deep_refresh_interval,
            last_deep_refresh: None,
            cached_values: None,
        }
    }

    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.collect_at(Instant::now(), Self::bluetooth_values)
    }

    fn collect_at(
        &mut self, now: Instant, probe: impl FnOnce(&HostOptions, Duration) -> Result<BTreeMap<String, Value>, String>,
    ) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("unavailable")),
            ("bluetooth_device_names".to_owned(), Value::Array(Vec::new())),
        ]);

        let refresh = self.last_deep_refresh.map(|last| now.duration_since(last) >= self.deep_refresh_interval).unwrap_or(true);
        if refresh {
            self.last_deep_refresh = Some(now);
            match probe(&self.options, Duration::from_millis(self.options.timing.bluetooth_probe_timeout_ms)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_refresh_interval_controls_native_probe_count() {
        let mut configuration = dokime::plugin::Configuration::declared::<crate::Settings>();
        configuration.settings.insert("bluetooth_deep_refresh_ms".into(), dokime::Value::from(250_u64));
        configuration.settings.insert("bluetooth_probe_timeout_ms".into(), dokime::Value::from(125_u64));
        let options = HostOptions::from(configuration.settings::<crate::Settings>().unwrap());
        let mut collector = BluetoothCollector::new(options);
        let opened = Instant::now();
        let mut probes = Vec::new();
        for elapsed in [0, 249, 250, 499, 500] {
            let sample = collector
                .collect_at(opened + Duration::from_millis(elapsed), |_, timeout| {
                    probes.push((elapsed, timeout));
                    Ok(BTreeMap::from([("bluetooth_adapter_count".into(), Value::Integer(probes.len() as i64))]))
                })
                .unwrap();
            let expected = match elapsed {
                0 | 249 => 1,
                250 | 499 => 2,
                _ => 3,
            };
            assert_eq!(sample.values.get("bluetooth_adapter_count"), Some(&Value::Integer(expected)));
        }
        assert_eq!(probes, vec![(0, Duration::from_millis(125)), (250, Duration::from_millis(125)), (500, Duration::from_millis(125))]);
    }
}

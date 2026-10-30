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

#[cfg(feature = "runtime")]
pub(crate) use defaults::BLUETOOTH_DEEP_REFRESH;

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

pub(crate) const LATENCY_HOST: &str = "1.1.1.1";

// An Active Wi-Fi Scan Disturbs The Link, So Scans Never Start Closer Together Than This.
pub(crate) const MINIMUM_SCAN_INTERVAL_S: u32 = 30;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub probes: Probes,
    pub cpu_load: crate::host::cpu::load::CpuLoadSettings,
}

#[dokime::section(label = "Collection")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Probes {
    #[setting(label = "Wi-Fi Adapter")]
    pub wifi_adapter_name: Option<String>,

    #[setting(label = "Bluetooth Adapter")]
    pub bluetooth_adapter_name: Option<String>,

    #[setting(label = "Latency Host", default = LATENCY_HOST)]
    pub latency_host: String,

    #[setting(label = "Minimum Scan Interval (s)", default = MINIMUM_SCAN_INTERVAL_S, min = 1, max = 3_600, step = 1)]
    pub minimum_scan_interval_s: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostOptions {
    pub wifi_adapter_name: Option<String>,
    pub bluetooth_adapter_name: Option<String>,
    pub latency_host: Option<String>,
    pub minimum_scan_interval: Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<Probes> for HostOptions {
    fn from(probes: Probes) -> Self {
        Self {
            wifi_adapter_name: probes.wifi_adapter_name.and_then(selected),
            bluetooth_adapter_name: probes.bluetooth_adapter_name.and_then(selected),
            latency_host: selected(probes.latency_host),
            minimum_scan_interval: Duration::from_secs(u64::from(probes.minimum_scan_interval_s)),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn selected(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
}

#[cfg(feature = "runtime")]
mod defaults {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports

    /* === Constants === */

    pub(crate) const BLUETOOTH_DEEP_REFRESH: std::time::Duration = std::time::Duration::from_millis(300_000);

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    /* === Private Functions === */

    /* === Tests === */
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_probe_options() {
        let mut configuration = dokime::plugin::Configuration::declared::<Settings>();
        configuration.settings.insert("wifi_adapter_name".into(), dokime::Value::from("  wlan-test  "));
        let settings = configuration.settings::<Settings>().unwrap();
        let options = HostOptions::from(settings.probes);
        assert_eq!(options.wifi_adapter_name.as_deref(), Some("wlan-test"));
        assert_eq!(options.bluetooth_adapter_name, None);
        assert_eq!(options.latency_host.as_deref(), Some(LATENCY_HOST));
        assert_eq!(options.minimum_scan_interval, Duration::from_secs(u64::from(MINIMUM_SCAN_INTERVAL_S)));
    }
}

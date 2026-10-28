/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows {
    pub(crate) const WIFI_SCAN_WINDOW: std::time::Duration = std::time::Duration::from_millis(4_000);
}
#[cfg(all(feature = "runtime", target_os = "windows"))]
pub(crate) use windows::WIFI_SCAN_WINDOW;

#[cfg(feature = "runtime")]
mod defaults {
    pub(crate) const PING_REPLY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2_000);
    pub(crate) const NETWORK_LINK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5_000);
    pub(crate) const BLUETOOTH_DEEP_REFRESH: std::time::Duration = std::time::Duration::from_millis(300_000);
}
#[cfg(feature = "runtime")]
pub(crate) use defaults::{BLUETOOTH_DEEP_REFRESH, NETWORK_LINK_INTERVAL, PING_REPLY_TIMEOUT};

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

pub(crate) const LATENCY_HOST: &str = "1.1.1.1";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub probes: Probes,
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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostOptions {
    pub wifi_adapter_name: Option<String>,
    pub bluetooth_adapter_name: Option<String>,
    pub latency_host: Option<String>,
}

/* === Enums === */

/* === Implementations === */

impl From<Probes> for HostOptions {
    fn from(probes: Probes) -> Self {
        Self {
            wifi_adapter_name: probes.wifi_adapter_name.and_then(selected),
            bluetooth_adapter_name: probes.bluetooth_adapter_name.and_then(selected),
            latency_host: selected(probes.latency_host),
        }
    }
}

/* === Private Functions === */

fn selected(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
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
    }
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod contracts;
mod rows;
mod table;

/* === Re-Exports === */

pub use rows::{Bluetooth, Network, System, WifiLatency, WifiLink, WifiScan};

pub use contracts::host_telemetry_contract;
pub use contracts::host_telemetry_contracts;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use dokime_domain::model::identity::Identifier;

/* === Constants === */

pub const HOST_STREAMS: &[HostTelemetryStream] = &[
    HostTelemetryStream::System,
    HostTelemetryStream::Network,
    HostTelemetryStream::WifiLink,
    HostTelemetryStream::WifiScan,
    HostTelemetryStream::WifiLatency,
    HostTelemetryStream::Bluetooth,
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unsupported host telemetry stream: {0}")]
pub struct UnknownHostStream(pub String);

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTelemetryStream {
    System,
    Network,
    WifiLink,
    WifiScan,
    WifiLatency,
    Bluetooth,
}

/* === Traits === */

/* === Implementations === */

impl HostTelemetryStream {
    // Default Cadence For The SDK's Compile-Time Observer Declarations.
    pub const fn interval(self) -> dokime_domain::model::time::DurationMillis {
        dokime_domain::model::time::DurationMillis(match self {
            Self::System => crate::settings::SYSTEM_INTERVAL_MS,
            Self::Network => crate::settings::NETWORK_INTERVAL_MS,
            Self::WifiLink => crate::settings::WIFI_LINK_INTERVAL_MS,
            Self::Bluetooth => crate::settings::BLUETOOTH_INTERVAL_MS,
            Self::WifiLatency => crate::settings::WIFI_LATENCY_INTERVAL_MS,
            Self::WifiScan => crate::settings::WIFI_SCAN_INTERVAL_MS,
        })
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "host.system",
            Self::Network => "host.network",
            Self::WifiLink => "host.wifi.link",
            Self::WifiScan => "host.wifi.scan",
            Self::WifiLatency => "host.wifi.latency",
            Self::Bluetooth => "host.bluetooth",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Host System",
            Self::Network => "Host Network",
            Self::WifiLink => "Host WiFi Link",
            Self::WifiScan => "Host WiFi Scan",
            Self::WifiLatency => "Host WiFi Latency",
            Self::Bluetooth => "Host Bluetooth",
        }
    }

    pub fn identifier(self) -> Identifier {
        identifier(self.as_str())
    }
}

impl TryFrom<&str> for HostTelemetryStream {
    type Error = UnknownHostStream;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "host.system" | "system" => Ok(Self::System),
            "host.network" | "network" => Ok(Self::Network),
            "host.wifi.link" | "wifi_link" => Ok(Self::WifiLink),
            "host.wifi.scan" | "wifi_scan" => Ok(Self::WifiScan),
            "host.wifi.latency" | "wifi_latency" | "latency" => Ok(Self::WifiLatency),
            "host.bluetooth" | "bluetooth" => Ok(Self::Bluetooth),
            other => Err(UnknownHostStream(other.to_owned())),
        }
    }
}

/* === Public Functions === */

pub fn host_stream_identifiers() -> Vec<Identifier> {
    HOST_STREAMS.iter().map(|stream| stream.identifier()).collect()
}

/* === Private Functions === */

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("built-in host telemetry identifiers are valid")
}

/* === Tests === */

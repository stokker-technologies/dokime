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
pub mod table;

/* === Re-Exports === */

pub use contracts::host_telemetry_contract;
pub use contracts::host_telemetry_contracts;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::Identifier;

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

product_error!(UnknownHostStream, formatter {
    Self(named) => write!(formatter, "unsupported host telemetry stream: {named}"),
});

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq)]
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
    // The One Source Of Each Host Stream's Default Cadence.
    pub const fn interval(self) -> crate::model::time::DurationMillis {
        crate::model::time::DurationMillis(match self {
            Self::System | Self::Network | Self::WifiLink => 1_000,
            Self::Bluetooth => 5_000,
            Self::WifiLatency => 10_000,
            Self::WifiScan => 60_000,
        })
    }

    pub fn as_str(self) -> &'static str {
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

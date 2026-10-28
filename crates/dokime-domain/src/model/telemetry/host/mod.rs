/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod contracts;
pub mod table;

pub use contracts::host_telemetry_contract;
pub use contracts::host_telemetry_contracts;
pub use contracts::{PreparedHostPayload, prepare_host_telemetry_payload};

use serde::Deserialize;
use serde::Serialize;

use crate::model::identity::Identifier;

pub const HOST_STREAMS: &[HostTelemetryStream] = &[
    HostTelemetryStream::System,
    HostTelemetryStream::Network,
    HostTelemetryStream::Wifi,
    HostTelemetryStream::WifiLatency,
    HostTelemetryStream::WifiChannelOccupancy,
    HostTelemetryStream::Bluetooth,
];

product_error!(UnknownHostStream, formatter {
    Self(named) => write!(formatter, "unsupported host telemetry stream: {named}"),
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownHostStream(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTelemetryStream {
    System,
    Network,
    Wifi,
    WifiLatency,
    WifiChannelOccupancy,
    Bluetooth,
}

impl HostTelemetryStream {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "host.system",
            Self::Network => "host.network",
            Self::Wifi => "host.wifi",
            Self::WifiLatency => "host.wifi.latency",
            Self::WifiChannelOccupancy => "host.wifi.channel_occupancy",
            Self::Bluetooth => "host.bluetooth",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Host System",
            Self::Network => "Host Network",
            Self::Wifi => "Host WiFi",
            Self::WifiLatency => "Host WiFi Latency",
            Self::WifiChannelOccupancy => "Host WiFi Network Occupancy",
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
            "host.wifi" | "wifi" => Ok(Self::Wifi),
            "host.wifi.latency" | "wifi_latency" | "latency" => Ok(Self::WifiLatency),
            "host.wifi.channel_occupancy" | "wifi_channel_occupancy" | "channel_occupancy" => Ok(Self::WifiChannelOccupancy),
            "host.bluetooth" | "bluetooth" => Ok(Self::Bluetooth),
            other => Err(UnknownHostStream(other.to_owned())),
        }
    }
}

pub fn host_stream_identifiers() -> Vec<Identifier> {
    HOST_STREAMS.iter().map(|stream| stream.identifier()).collect()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("built-in host telemetry identifiers are valid")
}

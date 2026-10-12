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

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::{HOST_STREAMS, HostTelemetryStream};
use dokime_contract::core::DurationMillis;

/* === Constants === */
pub const DEFAULT_HOST_INTERVAL: DurationMillis = DurationMillis(1_000);
pub const DEFAULT_NETWORK_LINK_INTERVAL: DurationMillis = DurationMillis(5_000);
pub const DEFAULT_WIFI_LATENCY_INTERVAL: DurationMillis = DurationMillis(10_000);
pub const DEFAULT_WIFI_SCAN_INTERVAL: DurationMillis = DurationMillis(30_000);
pub const DEFAULT_BLUETOOTH_INTERVAL: DurationMillis = DurationMillis(5_000);
pub const DEFAULT_BLUETOOTH_DEEP_SCAN_INTERVAL: DurationMillis = DurationMillis(300_000);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostTelemetryConfig {
    pub enabled: bool,
    pub enabled_streams: Vec<HostTelemetryStream>,
    pub interval: DurationMillis,
    pub source: HostTelemetrySourceKind,
    pub options: HostOptions,
    pub system_interval: DurationMillis,
    pub network_interval: DurationMillis,
    pub network_link_interval: DurationMillis,
    pub wifi_interval: DurationMillis,
    pub wifi_latency_interval: DurationMillis,
    pub wifi_scan_interval: DurationMillis,
    pub bluetooth_interval: DurationMillis,
    pub bluetooth_deep_scan_interval: DurationMillis,
    pub probe_timeout: DurationMillis,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostOptions {
    pub wifi_adapter_name: Option<String>,
    pub bluetooth_adapter_name: Option<String>,
    pub latency_host: Option<String>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTelemetrySourceKind {
    Native,
    Fixture,
}

/* === Traits === */

/* === Implementations === */
impl Default for HostTelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enabled_streams: Vec::new(),
            interval: DEFAULT_HOST_INTERVAL,
            source: HostTelemetrySourceKind::Native,
            options: HostOptions::default(),
            system_interval: DEFAULT_HOST_INTERVAL,
            network_interval: DEFAULT_HOST_INTERVAL,
            network_link_interval: DEFAULT_NETWORK_LINK_INTERVAL,
            wifi_interval: DEFAULT_NETWORK_LINK_INTERVAL,
            wifi_latency_interval: DEFAULT_WIFI_LATENCY_INTERVAL,
            wifi_scan_interval: DEFAULT_WIFI_SCAN_INTERVAL,
            bluetooth_interval: DEFAULT_BLUETOOTH_INTERVAL,
            bluetooth_deep_scan_interval: DEFAULT_BLUETOOTH_DEEP_SCAN_INTERVAL,
            probe_timeout: DurationMillis(10_000),
        }
    }
}

impl HostTelemetryConfig {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_source(mut self, source: HostTelemetrySourceKind) -> Self {
        self.source = source;
        self
    }

    pub fn with_enabled_streams(mut self, streams: Vec<HostTelemetryStream>) -> Self {
        self.enabled_streams = streams;
        self
    }

    pub fn task_streams(&self) -> Vec<HostTelemetryStream> {
        if self.enabled_streams.is_empty() {
            return HOST_STREAMS.to_vec();
        }
        self.enabled_streams.clone()
    }

    pub fn interval_for(&self, stream: HostTelemetryStream) -> DurationMillis {
        match stream {
            HostTelemetryStream::System => self.system_interval,
            HostTelemetryStream::Network => self.network_interval,
            HostTelemetryStream::Wifi => self.wifi_interval,
            HostTelemetryStream::WifiLatency => self.wifi_latency_interval,
            HostTelemetryStream::WifiChannelOccupancy => self.wifi_scan_interval,
            HostTelemetryStream::Bluetooth => self.bluetooth_interval,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

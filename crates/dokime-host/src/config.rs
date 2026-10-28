/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostTelemetryConfig {
    pub source: HostTelemetrySourceKind,
    pub options: HostOptions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostOptions {
    pub wifi_adapter_name: Option<String>,
    pub bluetooth_adapter_name: Option<String>,
    pub latency_host: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTelemetrySourceKind {
    #[default]
    Native,
    Fixture,
}

impl HostTelemetryConfig {
    pub fn with_source(mut self, source: HostTelemetrySourceKind) -> Self {
        self.source = source;
        self
    }
}

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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTelemetrySourceKind {
    #[default]
    Native,
    Fixture,
}

/* === Traits === */

/* === Implementations === */

impl HostTelemetryConfig {
    pub fn with_source(mut self, source: HostTelemetrySourceKind) -> Self {
        self.source = source;
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
use std::sync::Arc;
// Third Party Imports

// Local Imports
use crate::{HostTelemetryCollector, HostTelemetryConfig, telemetry};
use dokime_domain::model::plugins::InstalledPlugin;

/* === Constants === */
pub const HOST_SOURCE_ID: &str = "host";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn installed_plugin() -> InstalledPlugin {
    InstalledPlugin {
        source: HOST_SOURCE_ID.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        description: "Captures host telemetry streams on demand.".to_owned(),
        enabled: true,
        telemetry_contracts: telemetry::contracts(),
        ..Default::default()
    }
}

pub fn host_telemetry_collector(config: HostTelemetryConfig) -> Arc<dyn HostTelemetryCollector> {
    telemetry::collector(config)
}

pub fn default_host_telemetry_collector() -> Arc<dyn HostTelemetryCollector> {
    host_telemetry_collector(HostTelemetryConfig::default())
}

/* === Private Functions === */

/* === Tests === */

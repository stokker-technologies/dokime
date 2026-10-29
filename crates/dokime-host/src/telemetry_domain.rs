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

/// Installation Catalog Identity, Distinct From The Stored Sample Source.
pub const HOST_PLUGIN_SOURCE: &str = "host";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn installed_plugin() -> InstalledPlugin {
    InstalledPlugin {
        source: HOST_PLUGIN_SOURCE.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        description: "Captures host telemetry streams on demand.".to_owned(),
        enabled: true,
        telemetry_contracts: telemetry::contracts(),
        ..Default::default()
    }
}

/// Apply The Effective Instance Cadence Without Changing Sample Or Catalog Identity.
pub fn configure_contracts(installation: &mut InstalledPlugin) {
    if installation.source != HOST_PLUGIN_SOURCE {
        return;
    }
    for contract in &mut installation.telemetry_contracts {
        configure_contract(contract);
    }
}

pub fn host_telemetry_collector(config: HostTelemetryConfig) -> Arc<dyn HostTelemetryCollector> {
    telemetry::collector(config)
}

/* === Private Functions === */

fn configure_contract(contract: &mut dokime_domain::model::telemetry::TelemetryContract) {
    let stream = crate::HOST_STREAMS.iter().copied().find(|stream| {
        let identifier = stream.as_str();
        identifier == contract.stream.as_str()
    });
    if let Some(stream) = stream {
        let interval = telemetry::interval_ms(stream);
        contract.interval = Some(dokime_domain::model::time::DurationMillis(interval));
    }
}

/* === Tests === */

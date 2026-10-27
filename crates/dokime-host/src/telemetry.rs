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
use crate::{Host, HostOptions, HostTelemetryConfig, HostTelemetryStream, host_telemetry_contracts};
use dokime_domain::model::telemetry::TelemetryBatch;
use dokime_domain::model::telemetry::TelemetryContract;

/* === Constants === */

const SYSTEM_INTERVAL_MS: u64 = 1_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct HostTelemetryCollectorImpl {
    host: Host,
    options: HostOptions,
}

/* === Enums === */

/* === Traits === */

pub trait HostTelemetryCollector: Send + Sync + 'static {
    fn interval(&self, stream: &str) -> std::time::Duration;

    fn collect(&self, stream: &str) -> Result<Option<TelemetryBatch>, String>;
}

/* === Implementations === */

impl HostTelemetryCollectorImpl {
    fn new(config: HostTelemetryConfig) -> Self {
        let options = config.options.clone();
        Self { host: Host::new(config), options }
    }
}

impl HostTelemetryCollector for HostTelemetryCollectorImpl {
    fn interval(&self, stream: &str) -> std::time::Duration {
        HostTelemetryStream::try_from(stream)
            .map(|stream| std::time::Duration::from_millis(interval_ms(stream)))
            .unwrap_or(std::time::Duration::from_millis(SYSTEM_INTERVAL_MS))
    }

    fn collect(&self, stream: &str) -> Result<Option<TelemetryBatch>, String> {
        let stream = HostTelemetryStream::try_from(stream).map_err(|error| error.to_string())?;
        self.host.collect_stream_with_options(stream, &self.options).map_err(|error| error.to_string())
    }
}

/* === Public Functions === */

pub fn contracts() -> Vec<TelemetryContract> {
    host_telemetry_contracts()
}

pub fn collector(config: HostTelemetryConfig) -> Arc<dyn HostTelemetryCollector> {
    Arc::new(HostTelemetryCollectorImpl::new(config))
}

/* === Private Functions === */

pub(crate) fn interval_ms(stream: HostTelemetryStream) -> u64 {
    match stream {
        HostTelemetryStream::System | HostTelemetryStream::Network => 1_000,
        HostTelemetryStream::Wifi | HostTelemetryStream::Bluetooth => 5_000,
        HostTelemetryStream::WifiLatency => 10_000,
        HostTelemetryStream::WifiChannelOccupancy => 30_000,
    }
}

/* === Tests === */

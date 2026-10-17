/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod collectors;
mod config;
mod error;
mod fixture;
mod host;
pub(crate) mod platform;
mod sample;
mod tasks;
mod telemetry;
mod telemetry_domain;

/* === Re-Exports === */
// The stream set is a contract both ends compile; it lives in the api crate.
pub use config::{
    DEFAULT_BLUETOOTH_DEEP_SCAN_INTERVAL, DEFAULT_BLUETOOTH_INTERVAL, DEFAULT_HOST_INTERVAL,
    DEFAULT_NETWORK_LINK_INTERVAL, DEFAULT_WIFI_LATENCY_INTERVAL, DEFAULT_WIFI_SCAN_INTERVAL,
    HostOptions, HostTelemetryConfig, HostTelemetrySourceKind,
};
pub use dokime_contract::telemetry::{
    HOST_STREAMS, HostTelemetryStream, host_stream_identifiers, host_telemetry_contract,
    host_telemetry_contracts,
};
pub use error::{Error, Result};
pub use fixture::{FixtureHostTelemetrySource, HOST_FIXTURE_SOURCE};
pub use host::{Host, HostSnapshot};
pub use sample::HostStreamSample;

pub use tasks::HostTelemetryTaskSpec;
pub use telemetry::HostTelemetryCollector;
pub use telemetry_domain::{
    default_host_telemetry_collector, host_telemetry_collector, installed_plugin,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
pub const HOST_SOURCE_ID: &str = "host.telemetry";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn new_id<T>() -> T
where
    T: TryFrom<String>,
    T::Error: std::fmt::Debug,
{
    T::try_from(uuid::Uuid::now_v7().to_string())
        .expect("a freshly generated UUID is a valid Dokime identity")
}


/* === Tests === */

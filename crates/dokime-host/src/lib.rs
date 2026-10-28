/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod collectors;
mod config;
mod error;
mod fixture;
mod host;
pub(crate) mod platform;
mod sample;
mod telemetry;
mod telemetry_domain;

// The stream set is a contract both ends compile; it lives in the api crate.
pub use config::{HostOptions, HostTelemetryConfig, HostTelemetrySourceKind};
pub use dokime_domain::model::telemetry::HOST_STREAMS;
pub use dokime_domain::model::telemetry::HostTelemetryStream;
pub use dokime_domain::model::telemetry::host_stream_identifiers;
pub use dokime_domain::model::telemetry::host_telemetry_contract;
pub use dokime_domain::model::telemetry::host_telemetry_contracts;
pub use error::{Error, Result};
pub use fixture::{FixtureHostTelemetrySource, HOST_FIXTURE_SOURCE};
pub use host::{Host, HostSnapshot};
pub use sample::HostStreamSample;
pub use telemetry::{HostTelemetryCollector, collection_timeout};
pub use telemetry_domain::{HOST_PLUGIN_SOURCE, configure_contracts, host_telemetry_collector, installed_plugin};

/// Stored Sample Producer Identity, Distinct From The Installation Catalog Source.
pub const HOST_SOURCE_ID: &str = "host.telemetry";

fn new_id<T>() -> T
where
    T: TryFrom<String>,
    T::Error: std::fmt::Debug,
{
    T::try_from(uuid::Uuid::now_v7().to_string()).expect("a freshly generated UUID is a valid Dokime identity")
}

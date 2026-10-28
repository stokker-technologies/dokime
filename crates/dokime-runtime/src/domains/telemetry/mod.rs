/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod capture;
mod host;
pub(crate) mod leases;
pub(crate) mod output;
pub(crate) mod sampling;
mod streams;

pub(crate) use capture::Sampling;
pub use capture::{Sampler, SamplerSession, SettlementError};
pub use host::{HostTelemetryCollector, HostTelemetrySource};
pub use leases::LeaseFact;
pub(crate) use leases::StreamLeases;
#[cfg(test)]
pub(crate) use streams::admission::share_scopes;
pub(crate) use streams::admission::slots as telemetry_slots;
pub(crate) use streams::{Payloads, PreparedSample, TelemetryCursors};
pub use streams::{telemetry_stream_id, telemetry_stream_id_len};

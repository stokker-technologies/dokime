/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod capture;
pub(crate) mod leases;
pub(crate) mod output;
pub(crate) mod sampling;
mod streams;

/* === Re-Exports === */

pub(crate) use capture::Sampling;
pub use capture::{Sampler, SamplerSession, SettlementError};
pub(crate) use leases::StreamLeases;
pub use leases::{DeclaredLeases, LeaseFact, ViewerLeases};
#[cfg(test)]
pub(crate) use streams::admission::share_scopes;
pub(crate) use streams::admission::slots as telemetry_slots;
pub(crate) use streams::{Payloads, PreparedSample, TelemetryCursors};
pub use streams::{telemetry_stream_id, telemetry_stream_id_len};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

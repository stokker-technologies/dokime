/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod app;

#[cfg(feature = "runtime")]
pub mod collectors;
mod config;
mod error;
pub mod host;

#[cfg(feature = "runtime")]
pub(crate) mod platform;
mod sample;

/* === Re-Exports === */

#[cfg(feature = "runtime")]
pub use app::{HostPlugin, Workers};
pub use config::{HostOptions, Probes, Settings};
pub use dokime_domain::model::telemetry::{HOST_STREAMS, HostTelemetryStream, host_telemetry_contract, host_telemetry_contracts};
pub use error::{Error, Result};
pub use sample::HostStreamSample;

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

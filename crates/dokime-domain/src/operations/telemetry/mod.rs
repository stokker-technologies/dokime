/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod list_samples;
mod watch_samples;

/* === Re-Exports === */

pub use list_samples::ListSamples;
pub use watch_samples::WatchSamples;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::telemetry::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ListSamples(ListSamples),
    WatchSamples(WatchSamples),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for ListSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/ListSamples";
}

impl Endpoint for WatchSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/WatchSamples";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

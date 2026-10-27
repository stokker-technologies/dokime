/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod collect_diagnostics;
mod get_availability;
mod get_health;
mod get_identity;
mod watch_logs;
mod watch_trace;

/* === Re-Exports === */

pub use collect_diagnostics::CollectDiagnostics;
pub use get_availability::GetAvailability;
pub use get_health::GetHealth;
pub use get_identity::GetIdentity;
pub use watch_logs::WatchLogs;
pub use watch_trace::WatchTrace;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;
use super::TRANSFER;

use crate::model::backend::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    GetIdentity(GetIdentity),
    GetHealth(GetHealth),
    GetAvailability(GetAvailability),
    CollectDiagnostics(CollectDiagnostics),
    WatchLogs(WatchLogs),
    WatchTrace(WatchTrace),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for GetIdentity {
    const PATH: &'static str = "/dokime.v1.Backend/GetIdentity";
}

impl Endpoint for GetHealth {
    const PATH: &'static str = "/dokime.v1.Backend/GetHealth";
}

impl Endpoint for GetAvailability {
    const PATH: &'static str = "/dokime.v1.Backend/GetAvailability";
}

impl Endpoint for CollectDiagnostics {
    const PATH: &'static str = "/dokime.v1.Backend/CollectDiagnostics";
}

impl Endpoint for WatchLogs {
    const PATH: &'static str = "/dokime.v1.Backend/WatchLogs";
}

impl Endpoint for WatchTrace {
    const PATH: &'static str = "/dokime.v1.Backend/WatchTrace";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

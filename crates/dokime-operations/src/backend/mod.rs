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
use super::FOLLOW;
use super::ORDINARY;
use super::TRANSFER;

use dokime_domain::model::backend::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

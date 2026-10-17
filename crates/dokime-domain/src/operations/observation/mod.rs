/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod get_runtime_shell;
mod get_traceability;
mod watch_execution_progress;
mod watch_runtime;

/* === Re-Exports === */
pub use get_runtime_shell::GetRuntimeShell;
pub use get_traceability::GetTraceability;
pub use watch_execution_progress::WatchExecutionProgress;
pub use watch_runtime::WatchRuntime;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::wire::Metadata;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::execution::wire_cursor::Cursor;
#[allow(unused_imports)]
use crate::model::execution::wire::*;
use super::Access;
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

use crate::model::observation::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// This is a latest-state invalidation stream, not a durable event log. The
// initial frame always resynchronizes from current state. Subsequent frames may
// coalesce commits. Query cycle/catalog details after a frame as needed.
// A different epoch or a future position is rejected; reconnect without a cursor
// to explicitly resynchronize. Disconnect never cancels admitted execution.






















/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    WatchRuntime(WatchRuntime),
    WatchExecutionProgress(WatchExecutionProgress),
    GetRuntimeShell(GetRuntimeShell),
    GetTraceability(GetTraceability),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for WatchRuntime {
    const PATH: &'static str = "/dokime.v1.Observation/WatchRuntime";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for WatchExecutionProgress {
    const PATH: &'static str = "/dokime.v1.Observation/WatchExecutionProgress";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetRuntimeShell {
    const PATH: &'static str = "/dokime.v1.Observation/GetRuntimeShell";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetTraceability {
    const PATH: &'static str = "/dokime.v1.Observation/GetTraceability";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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

// Local Imports
use super::FOLLOW;
use super::ORDINARY;

use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::identity::wire::*;
use dokime_domain::model::observation::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

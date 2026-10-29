/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod get_runtime_shell;
mod get_traceability;
mod watch_execution_progress;
mod watch_runtime;

pub use get_runtime_shell::GetRuntimeShell;
pub use get_traceability::GetTraceability;
pub use watch_execution_progress::WatchExecutionProgress;
pub use watch_runtime::WatchRuntime;

use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use dokime_domain::model::execution::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::observation::wire::*;

#[harmos::catalog]
pub enum Operations {
    WatchRuntime(WatchRuntime),
    WatchExecutionProgress(WatchExecutionProgress),
    GetRuntimeShell(GetRuntimeShell),
    GetTraceability(GetTraceability),
}

impl Endpoint for WatchRuntime {
    const PATH: &'static str = "/dokime.v1.Observation/WatchRuntime";
}

impl Endpoint for WatchExecutionProgress {
    const PATH: &'static str = "/dokime.v1.Observation/WatchExecutionProgress";
}

impl Endpoint for GetRuntimeShell {
    const PATH: &'static str = "/dokime.v1.Observation/GetRuntimeShell";
}

impl Endpoint for GetTraceability {
    const PATH: &'static str = "/dokime.v1.Observation/GetTraceability";
}

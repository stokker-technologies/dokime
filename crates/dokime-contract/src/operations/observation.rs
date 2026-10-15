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
use serde::{Deserialize, Serialize};

// Local Imports
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
use super::cursor::Cursor;
#[allow(unused_imports)]
use super::execution::*;
use super::{Access, Endpoint, FOLLOW, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// This is a latest-state invalidation stream, not a durable event log. The
// initial frame always resynchronizes from current state. Subsequent frames may
// coalesce commits. Query cycle/catalog details after a frame as needed.
// A different epoch or a future position is rejected; reconnect without a cursor
// to explicitly resynchronize. Disconnect never cancels admitted execution.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchRuntimeRequest {
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStateFrame {
    pub cursor: Option<Cursor>,
    pub resynchronized: bool,
    pub shutting_down: bool,
    pub cycle_count: u64,
    // The slow work the backend is doing now, and work that ended moments ago.
    // A frame that changes only these repeats the previous cursor.
    #[row(
        description = "The slow work the backend is doing now, and work that ended moments ago. A frame that changes only these repeats the previous cursor."
    )]
    pub operations: Vec<OperationProgress>,
}

// One slow backend operation, such as rendering a report or sealing a cycle.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationProgress {
    pub operation_id: String,
    pub label: String,
    pub phase: String,
    // Absent while the operation cannot tell how far it has come.
    #[row(description = "Absent while the operation cannot tell how far it has come.")]
    pub fraction: Option<f64>,
    pub started_at: Option<Timestamp>,
    pub status: OperationStatus,
    pub finished_at: Option<Timestamp>,
    pub failure: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchExecutionProgressRequest {
    pub test_execution_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionProgress {
    pub test_execution_id: String,
    pub step_id: String,
    pub phase: String,
    pub completed: u64,
    pub total: u64,
    pub elapsed_ms: u64,
    pub remaining_ms: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionProgressFrame {
    pub revision: u64,
    pub resynchronized: bool,
    pub progress: Option<ExecutionProgress>,
    pub cleared: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetRuntimeShellRequest {
    // Zero selects the default page size (50); the maximum is 100.
    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,
    pub after_cycle_id: String,
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeShellSnapshot {
    pub session_id: String,
    pub revision: u64,
    pub shutting_down: bool,
    pub hub_metadata: Option<Metadata>,
    pub test_suite_count: u64,
    pub cycle_summaries: Vec<CycleSummary>,
    pub evidence_counts: Option<EvidenceCounts>,
    pub observed_cursor: Option<Cursor>,
    pub next_after_cycle_id: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceCounts {
    pub records: u64,
    pub telemetry: u64,
    pub prompts: u64,
    pub artifacts: u64,
    pub health: u64,
    pub traceability: u64,
    pub notes: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTraceabilityRequest {
    // Absent selects all current traceability groups. A supplied scope selects
    // that scope and its visible ancestors/global values.
    #[row(
        description = "Absent selects all current traceability groups. A supplied scope selects that scope and its visible ancestors/global values."
    )]
    pub scope: Option<EvidenceScope>,
    // Zero selects the default page size (50); the maximum is 100.
    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,
    // Continuations require the exact cursor returned by the preceding page.
    #[row(description = "Continuations require the exact cursor returned by the preceding page.")]
    pub after_traceability_id: String,
    pub cursor: Option<Cursor>,
    // Continuations must echo the host telemetry fingerprint returned by the
    // preceding page; an empty-host fingerprint is still nonempty.
    #[row(
        description = "Continuations must echo the host telemetry fingerprint returned by the preceding page; an empty-host fingerprint is still nonempty."
    )]
    pub host_traceability_fingerprint: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityPage {
    // The request scope is echoed so a consumer cannot accidentally apply a
    // response to another selected execution.
    #[row(
        description = "The request scope is echoed so a consumer cannot accidentally apply a response to another selected execution."
    )]
    pub scope: Option<EvidenceScope>,
    pub snapshots: Vec<TraceabilitySnapshot>,
    pub cursor: Option<Cursor>,
    pub next_after_traceability_id: String,
    pub complete: bool,
    // The source cut used for the host overlay. This binds continuations even
    // though host telemetry is stored outside the journal cursor.
    #[row(
        description = "The source cut used for the host overlay. This binds continuations even though host telemetry is stored outside the journal cursor."
    )]
    pub host_traceability_fingerprint: String,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Unspecified,
    Running,
    Finished,
    Failed,
}

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

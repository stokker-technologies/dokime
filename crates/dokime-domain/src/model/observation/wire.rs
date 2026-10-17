/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::execution::wire::*;
use crate::model::execution::wire_cursor::Cursor;
use crate::model::identity::wire::Metadata;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Types === */

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
    // Every phase entered, oldest first and the current one last; at most 16.
    #[row(description = "Every phase entered, oldest first and the current one last; at most 16.")]
    pub phases: Vec<String>,
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
pub struct TraceabilityPage {
    // The request scope is echoed so a consumer cannot accidentally apply a
    // response to another selected execution.
    #[row(description = "The request scope is echoed so a consumer cannot accidentally apply a response to another selected execution.")]
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

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Unspecified,
    Running,
    Finished,
    Failed,
}

/* === Public Functions === */

/* === Private Functions === */

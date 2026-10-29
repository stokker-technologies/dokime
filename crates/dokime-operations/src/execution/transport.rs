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
use std::time::Duration;

// Third Party Imports
use harmos::operations::OperationPolicy;

// Local Imports
use dokime_domain::model::time::Timestamp;

/* === Constants === */

pub const MAX_ARTIFACT_CHUNK_BYTES: usize = 1024 * 1024;

const EVIDENCE: OperationPolicy = crate::policy(Duration::from_secs(30));
const OPERATOR: OperationPolicy = crate::policy(Duration::from_secs(7 * 24 * 60 * 60));
const LEASE: OperationPolicy = crate::policy(Duration::from_secs(60));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The Execution Transport A Template Run Drives Against Its Host: Evidence
// Pages, Operator Questions And Telemetry Leases.

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Entry {
    #[row(description = "Numeric severity on the wire; producers use [`Level`].")]
    pub level: i32,

    pub message: String,

    #[row(description = "UTC RFC 3339 time at publication, with millisecond precision.")]
    pub timestamp: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct ExecutionProgress {
    #[row(description = "Author-facing description of the bounded operation.")]
    pub phase: String,

    #[row(description = "Work units whose loop bodies completed.")]
    pub completed: u64,

    #[row(description = "Work units the iterator contained when progress tracking began.")]
    pub total: u64,

    #[row(description = "Monotonic time since this operation began.")]
    pub elapsed_ms: u64,

    #[row(description = "Estimated monotonic time left, absent during estimator warm-up.")]
    pub remaining_ms: Option<u64>,
}

// The host's evidence admission: a sidecar pushes each drained page.
#[harmos::operation(resources = [crate::hosting::Host<EvidenceBatch>, harmos::operations::HostContext], response = EvidenceReceipt,
    id = "dokime.execution.evidence", policy = EVIDENCE, visibility = Public)]
#[derive(Clone, PartialEq)]
pub struct EvidenceBatch {
    #[row(description = "True only after observers, callbacks, cleanup and evaluation have settled.")]
    pub cleanup_settled: bool,

    #[row(description = "Opaque high-water mark acknowledging every item in this reply.")]
    pub cursor: u64,

    #[row(description = "Current authored phase identifier; an empty value means unchanged.")]
    pub phase: String,

    #[row(description = "Immutable completed callback invocations; host admission stamps ownership.")]
    pub step_attempts: Vec<String>,

    #[row(description = "Invocations waiting for host activation before their callbacks start.")]
    pub step_begins: Vec<String>,

    pub capture_phases: Vec<String>,

    #[row(description = "JSON-encoded `RecordPublishRequest` values whose scope is host-stamped.")]
    pub records: Vec<String>,

    #[row(description = "Terminal result, emitted as one cursor-addressed item after completion.")]
    pub outcome: Option<SidecarExecutionOutcome>,

    #[row(description = "Severity-bearing execution diagnostics; standard output remains Harmos IPC.")]
    pub diagnostics: Vec<Entry>,

    #[row(
        description = "Evidence files the execution produced, in bounded pieces.\nChunking bounds each item, but batching several bounded chunks can still exceed Harmos's complete encoded route frame. The workflow therefore pages accumulated chunks against the transport-owned reply budget, so one artifact routinely spans several replies. The host reassembles by transfer id and uploads once the terminal piece arrives. Cursor-addressed like the rest of the reply, so a retried drain re-delivers pieces the host has not acknowledged."
    )]
    pub artifact_chunks: Vec<ArtifactChunk>,

    #[row(
        description = "JSON-encoded operator prompts the execution is waiting on.\nFiled in order with the rest of the page; the run asks each one through [`AskOperator`] and waits for the answer rather than guessing."
    )]
    pub prompt_requests: Vec<String>,

    #[row(
        description = "JSON-encoded [`TraceabilitySection`](dokime_domain::model::traceability::TraceabilitySection) values the template stated before its first step.\nOne item per section, encoded like the samples, records and prompts beside it. The host stamps the scope, the time and the provenance source it files them under, because a sidecar naming its own provenance is a sidecar that can lie about it."
    )]
    pub traceability: Vec<String>,

    #[row(
        description = "JSON-encoded [`DocumentAddition`](crate::document::DocumentAddition) values a stage added to this execution's document.\nOne item per accepted addition, in the order the run accepted them, so the host merges them into the document it stored in the same order a replay of this evidence would. A refused addition never reaches here: the run narrated it into the execution log and changed nothing."
    )]
    pub documents: Vec<String>,

    #[row(
        description = "JSON-encoded [`LeaseRequest`](crate::telemetry::LeaseRequest) values a step raised with `ctx.lease::<T>()`.\nThe prompt pattern, for the same reason: a step asking for a lease is asking the host for something only the host can grant, and the answer comes back from [`LeaseStream`]."
    )]
    pub lease_requests: Vec<String>,

    #[row(
        description = "Lease tokens whose guard has been dropped, so the host may release them.\nA hold ends where the guard does, and the guard is a value in the step's own scope — so the release is an observation the run reports, not a request it makes. Repeating one is a successful no-op on the host side, which is what lets it ride the same replayable cursor as the rest."
    )]
    pub lease_releases: Vec<String>,

    #[row(description = "Latest live work-unit progress represented by this page.")]
    pub progress: Option<ExecutionProgress>,

    #[row(description = "Cleanup completed and all preceding output has drained; close inputs and acknowledge.")]
    pub record_subscriptions_settling: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct SidecarExecutionOutcome {
    pub passed: bool,
    pub message: Option<String>,

    #[row(
        description = "What each declared criterion decided. A sidecar that evaluates no criteria sends none, which is how it looked before this field existed."
    )]
    pub criteria: Vec<SidecarCriterionOutcome>,

    #[row(
        description = "The failure's whole story: a JSON-encoded [`Diagnostic`](crate::diagnostics::Diagnostic).\nJSON like the samples, records and prompts beside it on the drain reply: the drain wire carries typed payloads as encoded strings rather than mirroring every product shape as a protocol message of its own. Present only for a run that failed — a cancelled or passing run has no failure to detail."
    )]
    pub detail_json: Option<String>,

    #[row(
        description = "The stage that failed, as the run names it: a declared step's identifier, or `initialization`, `traceability`.\nThe hooks are not declared steps, so without this the host would have no row to put the failure's story on and a diagnostic from initialization would reach the console but never the document."
    )]
    pub failed_in: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct SidecarCriterionOutcome {
    #[row(
        description = "The criterion id as the suite declared it. One the suite never declared is refused rather than invented into the record."
    )]
    pub identifier: String,

    pub label: String,
    pub status: i32,

    #[row(
        description = "Why, for a criterion that failed or could not decide. This is where an indeterminate criterion records the error that stopped it."
    )]
    pub message: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct ArtifactChunk {
    #[row(description = "Replay-stable identity for this artifact within one execution.")]
    pub transfer_id: String,

    #[row(description = "Path-like name the artifact is filed under, stable across its pieces.")]
    pub name: String,

    #[row(description = "Media type, repeated so every replayed piece is independently checked.")]
    pub mime: String,

    #[row(description = "Zero-based position of this piece within the artifact.")]
    pub sequence: u64,

    pub bytes: Vec<u8>,

    #[row(description = "Whether this is the artifact's terminal piece.")]
    pub last: bool,

    #[row(
        description = "Whether that terminal piece abandons the artifact instead of completing it.\nAn aborting piece carries no bytes, sets `last`, and takes the transfer's next sequence number. It says the producer gave up, so the host releases what it assembled and the slot the transfer held among the execution's open artifacts, and never opens an upload."
    )]
    pub aborted: bool,

    #[row(
        description = "Whether this piece starts the artifact's content over.\nThe host discards what it assembled for this transfer and begins again from this piece's bytes, so the artifact it stores is the new content alone. A watched file that was rewritten rather than appended to is the one producer of this: a growing file sends what it grew by, and a file that no longer starts with what was already sent sends itself again. Never set together with `aborted`, which ends the transfer instead."
    )]
    pub replace: bool,
}

// Asks the operator through the host and waits for the answer.
#[harmos::operation(resources = [crate::hosting::Host<OperatorQuestion>, harmos::operations::HostContext], response = OperatorAnswer,
    id = "dokime.execution.prompt", policy = OPERATOR, visibility = Public)]
#[derive(Clone, PartialEq)]
pub struct OperatorQuestion {
    pub prompt_json: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct OperatorAnswer {
    pub response_json: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct EvidenceReceipt {
    #[row(description = "The host's answer for each step the page asked to begin.")]
    pub admitted: Vec<StepAdmission>,

    #[row(description = "Every record admitted through cleanup has been delivered.")]
    pub settled: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct StepAdmission {
    pub attempt_id: String,
    pub accepted: bool,
    pub message: String,
}

// Asks the host to grant a telemetry lease.
#[harmos::operation(resources = [crate::hosting::Host<LeaseAsk>, harmos::operations::HostContext], response = LeaseResolution,
    id = "dokime.execution.lease", policy = LEASE, visibility = Public)]
#[derive(Clone, PartialEq)]
pub struct LeaseAsk {
    pub lease_json: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct LeaseResolution {
    pub granted: bool,
    pub message: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/* === Traits === */

/* === Implementations === */

impl Entry {
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            level: level as i32,
            message: message.into(),
            timestamp: timestamp(),
        }
    }
}

impl TryFrom<i32> for Level {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, i32> {
        Ok(match value {
            0 => Self::Trace,
            1 => Self::Debug,
            2 => Self::Info,
            3 => Self::Warn,
            4 => Self::Error,
            unknown => return Err(unknown),
        })
    }
}

impl ArtifactChunk {
    pub fn split(transfer_id: impl Into<String>, name: impl Into<String>, mime: impl Into<String>, bytes: Vec<u8>) -> Vec<Self> {
        let transfer_id = transfer_id.into();
        let name = name.into();
        let mime = mime.into();
        if bytes.is_empty() {
            return vec![Self {
                transfer_id,
                name,
                mime,
                sequence: 0,
                bytes,
                last: true,
                aborted: false,
                replace: false,
            }];
        }
        let chunk_count = bytes.len().div_ceil(MAX_ARTIFACT_CHUNK_BYTES);
        bytes
            .chunks(MAX_ARTIFACT_CHUNK_BYTES)
            .enumerate()
            .map(|(sequence, bytes)| Self {
                transfer_id: transfer_id.clone(),
                name: name.clone(),
                mime: mime.clone(),
                sequence: sequence as u64,
                bytes: bytes.to_vec(),
                last: sequence + 1 == chunk_count,
                aborted: false,
                replace: false,
            })
            .collect()
    }
}

/* === Public Functions === */

pub fn timestamp() -> String {
    let now = Timestamp::from_system_time(std::time::SystemTime::now());
    format!("{}.{:03}Z", &now.as_str()[..19], now.unix_millis().unwrap_or_default() % 1000)
}

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(EvidenceBatch, OperatorQuestion, LeaseAsk);
}

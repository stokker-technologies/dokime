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
// Third-Party Imports
use harmos::operations::{
    OperationDeclaration, OperationPolicy, RestartPolicy, RetryPolicy, Visibility, WireOperation,
};
// Local Imports

/* === Constants === */
const EVIDENCE_BOUND: Duration = Duration::from_secs(30);

const OPERATOR_BOUND: Duration = Duration::from_secs(7 * 24 * 60 * 60);

const LEASE_BOUND: Duration = Duration::from_secs(60);

pub const RUN_BOUND: Duration = Duration::from_secs(30 * 24 * 60 * 60);

pub const SIGNAL_BOUND: Duration = Duration::from_secs(30);

pub const MAX_ARTIFACT_CHUNK_BYTES: usize = 1024 * 1024;

pub const MAX_EVIDENCE_PAGE_BYTES: usize = 2 * MAX_ARTIFACT_CHUNK_BYTES + 256 * 1024;

pub const MAX_ARTIFACT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/* === Macros === */
macro_rules! host_operation {
    ($(#[$meta:meta])* $name:ident, $request:ty, $response:ty, $id:literal, $timeout:expr) => {
        $(#[$meta])*
        pub struct $name;

        impl OperationDeclaration for $name {
            type Request = $request;
            type Response = $response;
            const ID: &'static str = $id;
            const POLICY: OperationPolicy = OperationPolicy {
                timeout: $timeout,
                attempt_timeout: $timeout,
                retries: RetryPolicy::Never,
                restart: RestartPolicy::Never,
            };
            const VISIBILITY: Visibility = Visibility::Shared;
        }

        impl WireOperation for $name {
            const VERSION: u32 = 1;
        }
    };
}

host_operation!(
    PushEvidence,
    EvidenceBatch,
    EvidenceReceipt,
    "dokime.execution.evidence",
    EVIDENCE_BOUND
);

host_operation!(
    AskOperator,
    OperatorQuestion,
    OperatorAnswer,
    "dokime.execution.prompt",
    OPERATOR_BOUND
);

host_operation!(
    LeaseStream,
    LeaseAsk,
    LeaseResolution,
    "dokime.execution.lease",
    LEASE_BOUND
);

/* === Type Aliases === */

/* === Structs === */
#[harmos::message(skip_debug)]
#[derive(Clone, PartialEq)]
pub struct RunExecution {
    pub execution_token: String,
    pub test_cycle_id: String,
    pub test_execution_id: String,
    pub template_identifier: String,
    pub workflow: String,
    pub parameters_json: String,
    #[row(
        description = "Host-resolved secrets for this owner. Never persisted as execution data."
    )]
    pub secrets_json: String,
    #[row(description = "Runner policy: automatic or interactive.")]
    pub mode: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct EvidenceBatch {
    #[row(
        description = "True only after observers, callbacks, cleanup and evaluation have settled."
    )]
    pub cleanup_settled: bool,
    #[row(description = "Opaque high-water mark acknowledging every item in this reply.")]
    pub cursor: u64,
    #[row(description = "Current authored phase identifier; an empty value means unchanged.")]
    pub phase: String,
    #[row(
        description = "Immutable completed callback invocations; host admission stamps ownership."
    )]
    pub step_attempts: Vec<String>,
    #[row(description = "Invocations waiting for host activation before their callbacks start.")]
    pub step_begins: Vec<String>,
    pub capture_phases: Vec<String>,
    #[row(description = "JSON-encoded `RecordPublishRequest` values whose scope is host-stamped.")]
    pub records: Vec<String>,
    #[row(description = "Terminal result, emitted as one cursor-addressed item after completion.")]
    pub outcome: Option<SidecarExecutionOutcome>,
    #[row(
        description = "Severity-bearing execution diagnostics; standard output remains Harmos IPC."
    )]
    pub diagnostics: Vec<crate::execution::log::Entry>,
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
    pub progress: Option<crate::execution::ExecutionProgress>,
    #[row(
        description = "Cleanup completed and all preceding output has drained; close inputs and acknowledge."
    )]
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

#[harmos::message]
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

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct ExecutionSignal {
    pub execution_token: String,
    #[row(description = "Cooperatively cancel the execution; idempotent.")]
    pub cancel: bool,
    #[row(description = "Admit or retry one interactive command.")]
    pub command: Option<CommandExecutionRequest>,
    #[row(description = "Hand down one admitted record a step may be waiting for.")]
    pub record: Option<DeliverRecordRequest>,
    #[row(description = "Hand down rows of a leased global stream.")]
    pub samples: Option<DeliverSamplesRequest>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct SignalReply {
    #[row(description = "The command's receipt, when the signal carried one.")]
    pub command: Option<CommandExecutionReply>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct DeliverRecordRequest {
    #[row(description = "JSON-encoded [`Record`](dokime_domain::model::records::evidence::Record), as the host filed it.")]
    pub record_json: String,
    #[row(description = "Execution scopes and origins frozen at admission, JSON-encoded.")]
    pub captured_scopes: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct DeliverSamplesRequest {
    #[row(
        description = "JSON-encoded [`TelemetrySample`](crate::telemetry::TelemetrySample) values, scope-stamped by the host with the run whose lease they answer."
    )]
    pub samples: Vec<String>,
    #[row(
        description = "JSON-encoded [`LeaseGap`](crate::telemetry::LeaseGap) values.\nThe absence of rows, on the same road as the rows, because it is the same fact about the same hold: the stream's owner went away and this stretch of the run has no measurements in it. A criterion reads them with `ctx.gaps::<T>()` rather than inferring a gap from a row count it has no expected value for."
    )]
    pub gaps: Vec<String>,
    #[row(
        description = "The scope the host captured these for, JSON-encoded.\nPresent so a sidecar running several executions can tell which of them a delivery is evidence for without reading it off every sample."
    )]
    pub scope_json: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct CommandExecutionRequest {
    pub execution_token: String,
    pub command_id: String,
    pub action: String,
    pub step: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct CommandExecutionReply {
    pub state: String,
    pub attempt_id: String,
    pub message: String,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum SidecarCriterionStatus {
    Unspecified = 0,
    Passed = 1,
    Failed = 2,
    Indeterminate = 3,
}

/* === Traits === */

/* === Implementations === */
impl std::fmt::Debug for RunExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunExecution")
            .field("execution_token", &self.execution_token)
            .field("test_execution_id", &self.test_execution_id)
            .field("template_identifier", &self.template_identifier)
            .field("workflow", &self.workflow)
            .field("mode", &self.mode)
            .field("secrets", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl ArtifactChunk {
    pub fn split(
        transfer_id: impl Into<String>,
        name: impl Into<String>,
        mime: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Vec<Self> {
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
/* === Private Functions === */

/* === Tests === */

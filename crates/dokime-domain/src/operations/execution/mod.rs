/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod append_planned_cases;
mod copy_cycle;
mod create_cycle;
mod delete_case;
mod get_case;
mod get_cycle;
mod get_execution;
mod get_execution_command;
mod get_run_request;
mod list_case_parameters;
mod list_cases;
mod list_cycles;
mod list_execution_parameters;
mod list_execution_steps;
mod list_executions;
mod list_step_criteria;
mod pause_cycle;
mod pause_execution;
mod rename_cycle;
mod rename_execution;
mod resume_cycle;
mod resume_execution;
mod run_case;
mod run_cycle;
mod run_execution;
mod schedule_case_rerun;
mod set_cycle_archived;
mod set_execution_archived;
mod stop_cycle;
mod stop_execution;
mod submit_execution_command;
mod update_cycle_policy;

/* === Re-Exports === */
pub use append_planned_cases::AppendPlannedCases;
pub use copy_cycle::CopyCycle;
pub use create_cycle::CreateCycle;
pub use delete_case::DeleteCase;
pub use get_case::GetCase;
pub use get_cycle::GetCycle;
pub use get_execution::GetExecution;
pub use get_execution_command::GetExecutionCommand;
pub use get_run_request::GetRunRequest;
pub use list_case_parameters::ListCaseParameters;
pub use list_cases::ListCases;
pub use list_cycles::ListCycles;
pub use list_execution_parameters::ListExecutionParameters;
pub use list_execution_steps::ListExecutionSteps;
pub use list_executions::ListExecutions;
pub use list_step_criteria::ListStepCriteria;
pub use pause_cycle::PauseCycle;
pub use pause_execution::PauseExecution;
pub use rename_cycle::RenameCycle;
pub use rename_execution::RenameExecution;
pub use resume_cycle::ResumeCycle;
pub use resume_execution::ResumeExecution;
pub use run_case::RunCase;
pub use run_cycle::RunCycle;
pub use run_execution::RunExecution;
pub use schedule_case_rerun::ScheduleCaseRerun;
pub use set_cycle_archived::SetCycleArchived;
pub use set_execution_archived::SetExecutionArchived;
pub use stop_cycle::StopCycle;
pub use stop_execution::StopExecution;
pub use submit_execution_command::SubmitExecutionCommand;
pub use update_cycle_policy::UpdateCyclePolicy;

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
use super::planning::*;
#[allow(unused_imports)]
use super::schema::*;
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Correlation is a caller label, never authenticated identity or an idempotency
// key. Only the run-specific idempotency_key fields support safe retries. Other
// mutations remain unsafe to replay after an ambiguous transport failure.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationContext {
    pub correlation_id: Option<String>,
}

// Copies the current plan, authored Definition/Operator cycle inputs, documents,
// and policy into a new ready cycle. No execution evidence or derived parameters
// are copied. The source remains untouched, including if currently running.
// Copy does not certify provider availability; Run validates that separately.




// Creates an empty Ready cycle. Cases join it only through AppendPlannedCases.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCycleReply {
    pub cycle_id: String,
    pub observed_cursor: Option<Cursor>,
}

// Append cases generated from the current registered catalog to an existing
// mutable cycle. The plan is resolved and validated against one captured
// registry snapshot before the existing append transaction is committed.




// Server-side selection and ordering for the live cycle list.
//
// `text` matches a cycle's label, description and identifier
// case-insensitively; an empty string selects everything. Repeated fields are
// disjunctive within themselves and conjunctive with the other fields, and an
// empty repeated field is no constraint. The timestamp range is inclusive and
// compares `created_at`.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleFilter {
    pub text: String,
    pub statuses: Vec<CycleStatus>,
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    // Absent lists only unarchived cycles.
    #[row(description = "Absent lists only unarchived cycles.")]
    pub archive: Option<ArchiveScope>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleOrder {
    pub key: CycleOrderKey,
    pub descending: bool,
}

// `filter` and `order` must be repeated unchanged on every continuation, and
// `after_cycle_id` must still be a member of the filtered result. A changed
// selection, a changed order, or a continuation ID the filter excludes is
// rejected rather than silently restarted.


#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCyclesReply {
    pub cycles: Vec<CycleSummary>,
    pub next_after_cycle_id: Option<String>,
    pub cursor: Option<Cursor>,
}





// Archiving is orthogonal to the lifecycle and admitted only on a terminal
// cycle or execution: it hides the row from lists that do not ask for archived
// rows and changes nothing else.




#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveReply {
    pub archived: bool,
}

// A rename is admitted until the subject seals: a sealed, archived or History
// cycle and an execution whose case has a sealed attempt refuse it with
// FailedPrecondition, since the sealed report froze the label. An execution's
// label is its case's label, so every attempt of that case is renamed together.




#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenameReply {
    // The label as stored, trimmed.
    #[row(description = "The label as stored, trimmed.")]
    pub label: String,
}

// DeleteCase only removes an unexecuted case. ScheduleCaseRerun makes the
// cycle ready without altering prior attempts or starting execution. A subsequent
// RunCase dispatches the next attempt; prior attempts and evidence remain retained.




// Optional keys are exact opaque values: 1–256 UTF-8 bytes, nonblank, no control
// characters. Namespace: stable node + authenticated principal, shared across
// RunCycle and RunCase. Reuse requires the identical cycle/case selection;
// correlation labels do not affect identity. Persist a key before first send.
// Repeating the original request resolves its durable receipt after restart;
// it never resumes an interrupted acquisition. Omission provides no retry safety.






// Lookup is independent of live cycles and epoch cursors. A Read principal may
// inspect its own receipts; Admin may inspect any. Unknown and other-owner IDs
// both return NotFound. A native receipt with no submitter is Admin-only.


// Commands belong to an admitted interactive run. Reuse the same command UUID
// and identical action to recover admission; a transport failure is uncertain.




// Completion references the actual callback attempt; Finish completes only
// after cleanup and evaluation. Interrupted commands are never replayed.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionCommandReceipt {
    pub request_id: String,
    pub command_id: String,
    pub execution_id: String,
    pub action: ExecutionCommandAction,
    pub step_id: Option<String>,
    pub phase: ExecutionCommandPhase,
    pub attempt_id: Option<String>,
    pub accepted_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub detail: Option<String>,
}

// Primary-published operation progress, not the current in-memory cycle state.
// finished_count counts folded outcomes; terminal Completed/Failed/Stopped also
// certifies required execution checkpoints settled. Interrupted means recovery
// did not resume device actions and cannot certify those terminal checkpoints.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRequestReceipt {
    pub request_id: String,
    pub node_id: String,
    pub cycle_id: String,
    pub phase: RunRequestPhase,
    pub paused: bool,
    pub stop_requested: bool,
    pub planned_count: u64,
    pub prepared_count: u64,
    pub finished_count: u64,
    pub unstarted_count: u64,
    pub failed_count: u64,
    pub claimed_count: u64,
    pub last_claim: Option<RunAttemptClaim>,
    pub accepted_at: Option<Timestamp>,
    // Authorized provider dispatch, not proof that a device action occurred.
    #[row(description = "Authorized provider dispatch, not proof that a device action occurred.")]
    pub provider_started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub submitted_by: Option<String>,
    // Why the request ended without its normal outcome: the interruption of an
    // Interrupted request, or the permanent cause a Failed request's final
    // reports were not sealed.
    #[row(
        description = "Why the request ended without its normal outcome: the interruption of an Interrupted request, or the permanent cause a Failed request's final reports were not sealed."
    )]
    pub interruption: Option<String>,
    pub mode: ExecutionMode,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RunAttemptClaim {
    pub ordinal: u64,
    pub case_id: String,
    pub execution_id: String,
}

// Observation position, not an exact command commit/checkpoint receipt.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlReply {
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RunReply {
    pub outcome: Option<RunReplyOutcome>,
}

// Admission or resolution of the original keyed request; not execution completion.
// command_id is the durable GetRunRequest request_id. A resolved job_id after
// restart does not imply a live job. Query the receipt for durable completion.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Admission {
    pub command_id: String,
    pub job_id: Option<String>,
    pub cursor: Option<Cursor>,
    pub accepted_at: Option<Timestamp>,
}

// Domain failures pack this in google.rpc.Status.details as Any with type URL
// type.googleapis.com/dokime.v1.Rejection. An error may follow
// a committed control transition (e.g. checkpoint failure); it does not promise
// rollback or make a retry safe.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Rejection {
    pub code: String,
    pub message: String,
    pub issues: Vec<ValidationIssue>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
    pub expected: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleSummary {
    pub cycle_id: String,
    pub metadata: Option<Metadata>,
    pub status: CycleStatus,
    pub case_count: u64,
    pub execution_count: u64,
    pub continue_on_failure: bool,
    pub observed_cursor: Option<Cursor>,
    pub created_at: Option<Timestamp>,
    pub started_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub result: Option<ResultSummary>,
    pub archived: bool,
    // The cycle report is sealed: its label no longer changes.
    #[row(description = "The cycle report is sealed: its label no longer changes.")]
    pub sealed: bool,
    // Planned cases by their latest attempt: running counts cases whose attempt
    // is running; queued counts cases still waiting to run (no attempt yet, an
    // undispatched attempt, or one admitted but not started).
    #[row(
        description = "Planned cases by their latest attempt: running counts cases whose attempt is running; queued counts cases still waiting to run (no attempt yet, an undispatched attempt, or one admitted but not started)."
    )]
    pub running_count: u64,
    pub queued_count: u64,
}

// Detail pages preserve native planned-case / execution-attempt vector order.
// Ordinals are zero-based positions at this cursor, not permanent identities.
// Continuations require the same epoch/revision and an existing after-ID in the
// requested cycle; changed revisions abort and unknown markers are invalid.


#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCasesReply {
    pub cases: Vec<CaseSummary>,
    pub next_after_case_id: Option<String>,
    pub cursor: Option<Cursor>,
}



#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionsReply {
    pub executions: Vec<ExecutionSummary>,
    pub next_after_execution_id: Option<String>,
    pub cursor: Option<Cursor>,
}

// One planned case with everything a navigator renders about it, so a shell
// never joins this page against ListExecutions to learn a case's state.
//
// The latest attempt is the last attempt materialized for this case, which is
// the same attempt the runtime's rerun and restart admission reads. `status`
// projects that attempt: no attempt at all is PLANNED; an attempt materialized
// but never dispatched is QUEUED; otherwise the attempt's own outcome.
// `attempt_count` counts every live attempt, including superseded ones.
// `rerun_scheduled` means this case already carries a recorded outcome and a
// fresh undispatched attempt on top of it, so the next RunCase runs that one.
// Historical projections report no actions: a checkpoint is immutable. A
// captured cycle grouping also retains the plan without its attempt records,
// so its rows carry CASE_STATUS_UNSPECIFIED and a zero attempt count; the
// checkpoint's own execution and `HistoricalCycleContext.execution_order` are
// where that cycle's attempts live.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseSummary {
    pub case_id: String,
    pub template_id: String,
    pub metadata: Option<Metadata>,
    pub ordinal: u64,
    pub status: CaseStatus,
    pub latest_execution_id: Option<String>,
    pub latest_execution_status: Option<ExecutionStatus>,
    pub attempt_count: u32,
    pub rerun_scheduled: bool,
    pub actions: Option<CaseActions>,
    pub suite_id: String,
}

// Which case mutations the runtime would admit right now, computed from the
// same checks RunCase, ScheduleCaseRerun and DeleteCase
// enforce. `unavailable_reasons` carries each refused operation's own reason
// text, deduplicated; it is diagnostic copy, not a stable code vocabulary.
// Eligibility is observed, never a promise: admission is re-checked on the
// mutation itself and a concurrent change can still reject it.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseActions {
    pub run: bool,
    pub schedule_rerun: bool,
    pub delete: bool,
    pub unavailable_reasons: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: String,
    pub case_id: String,
    pub status: ExecutionStatus,
    pub ordinal: u64,
    pub archived: bool,
    // The execution report is sealed: its case label no longer changes.
    #[row(description = "The execution report is sealed: its case label no longer changes.")]
    pub sealed: bool,
}

// Current-runtime detail. Published historical checkpoints are a separate API.


#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseDetail {
    pub summary: Option<CaseSummary>,
    pub suite: Option<Metadata>,
    pub parameter_count: u64,
    pub generation_index: u64,
    pub observed_cursor: Option<Cursor>,
}



#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionDetail {
    pub summary: Option<ExecutionSummary>,
    pub cycle_id: String,
    pub scope: Option<EvidenceScope>,
    pub admitted_suite: Option<Metadata>,
    pub admitted_template_id: Option<String>,
    pub admitted_template: Option<Metadata>,
    pub admitted_plugin: Option<ExecutionPluginIdentity>,
    pub invalidation: Option<ExecutionInvalidation>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<ResultSummary>,
    pub input_parameter_count: u64,
    pub output_parameter_count: u64,
    pub step_count: u64,
    pub observed_cursor: Option<Cursor>,
    pub controls: Option<ExecutionControls>,
}

// Snapshot eligibility reuses the mutation checks. A later mutation validates
// again; absent controls means no active execution-addressed request.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionControls {
    pub request_id: String,
    pub paused: bool,
    pub stop_requested: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_stop: bool,
    pub mode: ExecutionMode,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPluginIdentity {
    pub source: String,
    pub version: Option<String>,
    pub implementation: Option<ExecutionPluginIdentityImplementation>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionInvalidation {
    pub previous: Option<ExecutionPluginIdentity>,
    pub current: Option<ExecutionPluginIdentity>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultSummary {
    pub status: ResultStatus,
    pub passed: u32,
    pub failed: u32,
    pub blocked: u32,
    pub skipped: u32,
}

// A continuation belongs to exactly one parameter selection at one runtime
// revision. Names use ascending UTF-8 order. A changed revision requires a fresh
// first page; changing selection is rejected even if its after-name exists.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedCaseParameters {
    pub cycle_id: String,
    pub case_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterCursor {
    pub observed: Option<Cursor>,
    pub selection: Option<ParameterCursorSelection>,
}



#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseParameter {
    pub name: String,
    pub value: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCaseParametersReply {
    pub parameters: Vec<CaseParameter>,
    pub next_after_name: Option<String>,
    pub cursor: Option<ParameterCursor>,
}



#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMaterializedParametersReply {
    pub parameters: Vec<MaterializedParameter>,
    pub next_after_name: Option<String>,
    pub cursor: Option<ParameterCursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializedParameter {
    pub name: String,
    pub metadata: Option<Metadata>,
    pub group_id: Option<String>,
    pub source: ParameterSource,
    pub value: Option<Value>,
    pub unit: Option<MeasurementUnit>,
}

// Current execution evidence in native vector order. Repeated measurement names
// remain separate entries; ordinals belong to this revision, not stable identity.
// Continuation binds execution, step, collection and exact runtime revision.
// Any state change aborts continuation; restart with no cursor. Limits: default
// 50, maximum 100. A value exceeding the documented Value budget is an explicit
// ResourceExhausted error, never truncated evidence.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StepPageCursor {
    pub revision: Option<Cursor>,
    pub execution_id: String,
    pub step_id: String,
    pub kind: StepPageKind,
    pub next_ordinal: u64,
}





#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionStepsReply {
    pub steps: Vec<ExecutionStep>,
    pub next_cursor: Option<StepPageCursor>,
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListStepCriteriaReply {
    pub criteria: Vec<StepCriterion>,
    pub next_cursor: Option<StepPageCursor>,
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: String,
    pub metadata: Option<Metadata>,
    pub status: StepStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub measurement_count: u64,
    pub criterion_count: u64,
    pub ordinal: u64,
    // Why this step failed, for the one step that did. Present only on a failed
    // step, and already redacted: the step is the row an event log renders with a
    // status, so it is where the failure's own story belongs.
    #[row(
        description = "Why this step failed, for the one step that did. Present only on a failed step, and already redacted: the step is the row an event log renders with a status, so it is where the failure's own story belongs."
    )]
    pub detail: Option<Diagnostic>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StepMeasurement {
    pub metadata: Option<Metadata>,
    pub value: Option<Value>,
    pub unit: Option<MeasurementUnit>,
    pub ordinal: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StepCriterion {
    pub criterion_id: String,
    pub metadata: Option<Metadata>,
    pub status: CriterionStatus,
    pub observed: Option<Value>,
    pub message: Option<String>,
    pub ordinal: u64,
}

/* === Enums === */
// The requested order key. Ordering is always total: the key, then the cycle ID
// ascending, so equal keys never permute between pages. An unspecified key is
// UPDATED_AT. LABEL compares the case-folded label.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleOrderKey {
    Unspecified,
    UpdatedAt,
    CreatedAt,
    Label,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Automatic,
    Interactive,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionCommandAction {
    Unspecified,
    Next,
    RunStep,
    Finish,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionCommandPhase {
    Unspecified,
    Accepted,
    Dispatching,
    Running,
    Completed,
    Refused,
    Failed,
    Interrupted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunRequestPhase {
    Unspecified,
    Queued,
    Preparing,
    Running,
    Finishing,
    Completed,
    Failed,
    Stopped,
    Interrupted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleStatus {
    Unspecified,
    Ready,
    Running,
    Paused,
    Stopping,
    Completed,
    Failed,
    Blocked,
    Stopped,
}

// Which rows a list admits by their archive mark.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveScope {
    Unarchived,
    Archived,
    All,
}

// A case's state, derived from its latest attempt. This is a projection of the
// live attempt vector, not a stored field, so it changes with the attempts.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    Unspecified,
    Skipped,
    Invalidated,
    Planned,
    Queued,
    Running,
    Passed,
    Failed,
    Blocked,
    Cancelled,
    Interrupted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Unspecified,
    Invalidated,
    Pending,
    Running,
    Passed,
    Failed,
    Blocked,
    Cancelled,
    Interrupted,
    Skipped,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultStatus {
    Unspecified,
    Pending,
    Passed,
    Failed,
    Blocked,
    Cancelled,
}

// Input parameters are frozen at admission. Outputs are separate evidence and
// overlay inputs by key for presentation; they never rewrite the input snapshot.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionParameterSet {
    Unspecified,
    Inputs,
    Outputs,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterSource {
    Unspecified,
    Definition,
    Traceability,
    Environment,
    Resource,
    Operator,
    Runtime,
    Telemetry,
    Record,
    Output,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepPageKind {
    Unspecified,
    Steps,
    Measurements,
    Criteria,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Unspecified,
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Cancelled,
}

// Indeterminate means no conclusive verdict is available, including unreported
// or unevaluated criteria. This status alone does not prove evaluation ran.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriterionStatus {
    Unspecified,
    Pending,
    Passed,
    Failed,
    Indeterminate,
    Skipped,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RunReplyOutcome {
    Accepted(Admission),
    Rejected(Rejection),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionPluginIdentityImplementation {
    ContentDigest(String),
    Generation(String),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterCursorSelection {
    ExecutionInputsId(String),
    ExecutionOutputsId(String),
    PlannedCase(PlannedCaseParameters),
}

#[harmos::catalog]
pub enum Operations {
    ListExecutionSteps(ListExecutionSteps),
    ListStepCriteria(ListStepCriteria),
    CopyCycle(CopyCycle),
    CreateCycle(CreateCycle),
    GetCycle(GetCycle),
    GetCase(GetCase),
    GetExecution(GetExecution),
    ListCaseParameters(ListCaseParameters),
    ListExecutionParameters(ListExecutionParameters),
    ListCycles(ListCycles),
    ListCases(ListCases),
    ListExecutions(ListExecutions),
    AppendPlannedCases(AppendPlannedCases),
    RunCycle(RunCycle),
    RunCase(RunCase),
    RunExecution(RunExecution),
    PauseExecution(PauseExecution),
    ResumeExecution(ResumeExecution),
    StopExecution(StopExecution),
    GetRunRequest(GetRunRequest),
    SubmitExecutionCommand(SubmitExecutionCommand),
    GetExecutionCommand(GetExecutionCommand),
    PauseCycle(PauseCycle),
    ResumeCycle(ResumeCycle),
    StopCycle(StopCycle),
    DeleteCase(DeleteCase),
    ScheduleCaseRerun(ScheduleCaseRerun),
    SetCycleArchived(SetCycleArchived),
    SetExecutionArchived(SetExecutionArchived),
    RenameCycle(RenameCycle),
    RenameExecution(RenameExecution),
    UpdateCyclePolicy(UpdateCyclePolicy),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListExecutionSteps {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionSteps";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListStepCriteria {
    const PATH: &'static str = "/dokime.v1.Execution/ListStepCriteria";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for CopyCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CopyCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for CreateCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CreateCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetCycle {
    const PATH: &'static str = "/dokime.v1.Execution/GetCycle";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetCase {
    const PATH: &'static str = "/dokime.v1.Execution/GetCase";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetExecution {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecution";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCaseParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListCaseParameters";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListExecutionParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionParameters";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCycles {
    const PATH: &'static str = "/dokime.v1.Execution/ListCycles";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCases {
    const PATH: &'static str = "/dokime.v1.Execution/ListCases";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListExecutions {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutions";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for AppendPlannedCases {
    const PATH: &'static str = "/dokime.v1.Execution/AppendPlannedCases";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunCycle {
    const PATH: &'static str = "/dokime.v1.Execution/RunCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunCase {
    const PATH: &'static str = "/dokime.v1.Execution/RunCase";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunExecution {
    const PATH: &'static str = "/dokime.v1.Execution/RunExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for PauseExecution {
    const PATH: &'static str = "/dokime.v1.Execution/PauseExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ResumeExecution {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for StopExecution {
    const PATH: &'static str = "/dokime.v1.Execution/StopExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetRunRequest {
    const PATH: &'static str = "/dokime.v1.Execution/GetRunRequest";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for SubmitExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/SubmitExecutionCommand";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecutionCommand";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for PauseCycle {
    const PATH: &'static str = "/dokime.v1.Execution/PauseCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ResumeCycle {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for StopCycle {
    const PATH: &'static str = "/dokime.v1.Execution/StopCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for DeleteCase {
    const PATH: &'static str = "/dokime.v1.Execution/DeleteCase";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ScheduleCaseRerun {
    const PATH: &'static str = "/dokime.v1.Execution/ScheduleCaseRerun";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for SetCycleArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetCycleArchived";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for SetExecutionArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetExecutionArchived";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RenameCycle {
    const PATH: &'static str = "/dokime.v1.Execution/RenameCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RenameExecution {
    const PATH: &'static str = "/dokime.v1.Execution/RenameExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for UpdateCyclePolicy {
    const PATH: &'static str = "/dokime.v1.Execution/UpdateCyclePolicy";
    const ACCESS: Access = Access::Control;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

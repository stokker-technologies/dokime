/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod browse_executions;
mod delete_execution;
mod download_checkpoint_artifact;
mod get_checkpoint_context;
mod get_checkpoint_document;
mod get_checkpoint_evidence;
mod get_execution_checkpoint;
mod list_checkpoint_records;
mod list_checkpoint_telemetry;
mod read_checkpoint_output;

/* === Re-Exports === */
pub use browse_executions::BrowseExecutions;
pub use delete_execution::DeleteExecution;
pub use download_checkpoint_artifact::DownloadCheckpointArtifact;
pub use get_checkpoint_context::GetCheckpointContext;
pub use get_checkpoint_document::GetCheckpointDocument;
pub use get_checkpoint_evidence::GetCheckpointEvidence;
pub use get_execution_checkpoint::GetExecutionCheckpoint;
pub use list_checkpoint_records::ListCheckpointRecords;
pub use list_checkpoint_telemetry::ListCheckpointTelemetry;
pub use read_checkpoint_output::ReadCheckpointOutput;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::appearance::*;
#[allow(unused_imports)]
use super::catalog::*;
#[allow(unused_imports)]
use super::checkpoint::*;
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
#[allow(unused_imports)]
use super::content::*;
#[allow(unused_imports)]
use super::documents::*;
#[allow(unused_imports)]
use super::evidence::*;
#[allow(unused_imports)]
use super::execution::*;
#[allow(unused_imports)]
use super::notes::*;
#[allow(unused_imports)]
use super::plots::*;
#[allow(unused_imports)]
use super::prompts::*;
#[allow(unused_imports)]
use super::schema::*;
#[allow(unused_imports)]
use super::telemetry::*;
use super::value::Value;
use super::{Access, Endpoint, ORDINARY, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetExecutionCheckpointRequest {
    pub execution_identifier: String,
}

// Permanently delete one sealed historical execution. The caller supplies a
// stable request identifier so a retry can identify the accepted deletion.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExecutionRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub request_identifier: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExecutionResponse {
    pub request_identifier: String,
    pub execution_identifier: String,
    pub local_complete: bool,
}

// Server-side selection and ordering for the merged execution browse.
//
// `text` matches an entry's title, its execution identifier and its suite
// label case-insensitively; an empty string selects everything. Repeated
// fields are disjunctive within themselves and conjunctive with the other
// fields, and an empty repeated field is no constraint. The timestamp range is
// inclusive and compares `started_at`; an entry that has not started is
// excluded by either bound.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFilter {
    pub text: String,
    pub statuses: Vec<ExecutionStatus>,
    pub template_ids: Vec<String>,
    pub cycle_ids: Vec<String>,
    pub started_after: Option<Timestamp>,
    pub started_before: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub key: ExecutionOrderKey,
    pub descending: bool,
}

// `filter` and `order` must be repeated unchanged on every continuation, and
// `after_execution_id` must still be a member of the filtered result. A
// changed selection, a changed order, or a continuation ID the filter excludes
// is rejected rather than silently restarted.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowseExecutionsRequest {
    pub limit: u32,
    pub after_execution_id: String,
    pub filter: Option<ExecutionFilter>,
    pub order: Option<ExecutionOrder>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBrowsePage {
    pub entries: Vec<ExecutionBrowseEntry>,
    pub next_after_execution_id: Option<String>,
    pub discovery_complete: bool,
    pub conflicts: Vec<String>,
    pub omitted_conflicts: u32,
    pub issues: Vec<HistoryIssue>,
    pub omitted_issues: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBrowseEntry {
    pub metadata: Option<HistoricalExecutionMetadata>,
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub availability: ExecutionAvailability,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalExecutionMetadata {
    pub execution_id: String,
    pub cycle_id: Option<String>,
    pub case_id: String,
    pub title: String,
    pub status: ExecutionStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<ResultSummary>,
    // The frozen case template and admitted suite label this entry belongs to,
    // so a client can read back what `ExecutionFilter` selected on. Absent when
    // the stored row no longer resolves its case or suite revision.
    #[row(
        description = "The frozen case template and admitted suite label this entry belongs to, so a client can read back what `ExecutionFilter` selected on. Absent when the stored row no longer resolves its case or suite revision."
    )]
    pub template_id: Option<String>,
    pub suite_label: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryIssue {
    pub source: Option<String>,
    pub dataset: Option<String>,
    pub message: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointDocumentRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointDocument {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub document: Option<HistoricalExecutionDocument>,
    pub acquisition_finished: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalExecutionDocument {
    pub cycle: Option<HistoricalExecutionCycle>,
    pub execution: Option<HistoricalTestExecution>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalExecutionCycle {
    pub cycle_id: String,
    pub metadata: Option<Metadata>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalTestExecution {
    pub execution_id: String,
    pub case_id: String,
    pub suite: Option<Metadata>,
    pub status: ExecutionStatus,
    pub scope: Option<EvidenceScope>,
    pub snapshot: Option<HistoricalExecutionSnapshot>,
    pub output_parameters: Vec<MaterializedParameter>,
    pub documents: Vec<HistoricalDocument>,
    pub health: Vec<HealthItem>,
    pub notes: Vec<OperatorNote>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub steps: Vec<HistoricalStep>,
    pub result: Option<ResultSummary>,
    pub invalidation: Option<ExecutionInvalidation>,
    pub step_attempts: Vec<HistoricalStepAttempt>,
    pub active_step_attempt: Option<HistoricalStepAttemptStart>,
    pub mode: ExecutionMode,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalStepAttemptStart {
    pub attempt_id: String,
    pub step_id: String,
    pub started_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalStepAttempt {
    pub attempt_id: String,
    pub step_id: String,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub termination: StepTermination,
    pub detail: Option<Diagnostic>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalExecutionSnapshot {
    pub suite: Option<Metadata>,
    pub template_id: Option<String>,
    pub template: Option<Metadata>,
    pub plugin: Option<ExecutionPluginIdentity>,
    pub parameter_definitions: Vec<ParameterSchema>,
    pub parameters: Vec<MaterializedParameter>,
    pub documents: Vec<HistoricalDocument>,
    // Captured at admission; never reconstructed from the current suite catalog.
    #[row(
        description = "Captured at admission; never reconstructed from the current suite catalog."
    )]
    pub case_definition: Option<HistoricalCaseDefinition>,
    pub host_version: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalCaseDefinition {
    pub case_id: String,
    pub template_id: String,
    pub metadata: Option<Metadata>,
    pub suite: Option<Metadata>,
    pub parameters: Vec<CaseParameter>,
    pub generation_index: u64,
    pub suite_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalStep {
    pub step_id: String,
    pub metadata: Option<Metadata>,
    pub status: StepStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub measurements: Vec<StepMeasurement>,
    pub criteria: Vec<StepCriterion>,
}

// A historical document retains the standard presentation header and the
// authored layout/card identities. Payload data remains owned by the relevant
// evidence APIs, as it is for current DocumentFrame streams.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalDocument {
    pub presentation: Option<DocumentHeader>,
    pub layout_nodes: Vec<DocumentLayoutNode>,
    pub cards: Vec<HistoricalDocumentCard>,
    pub table_rows: Vec<DocumentTableRow>,
}

// Historical cards carry authored definitions in addition to the shared
// presentation/layout vocabulary. A checkpoint must remain renderable after
// the live suite registry has changed, so plot and summary configuration is
// explicit rather than reduced to an owner reference or fabricated defaults.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalDocumentCard {
    pub metadata: Option<Metadata>,
    pub filters: Vec<HistoricalDocumentFilter>,
    pub body: Option<HistoricalDocumentCardBody>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalTelemetrySummary {
    pub groups: Vec<HistoricalSummaryGroup>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalSummaryGroup {
    pub metadata: Option<Metadata>,
    pub tiles: Vec<HistoricalSummaryTile>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalSummaryTile {
    pub metadata: Option<Metadata>,
    pub value: Option<DataExpression>,
    pub filters: Vec<HistoricalDocumentFilter>,
    pub stat: DocumentSummaryStat,
    pub format: DocumentSummaryFormat,
    pub presentation: DocumentSummaryPresentation,
    pub unit: Option<String>,
    pub conditions: Vec<HistoricalSummaryCondition>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalSummaryCondition {
    pub operator: FilterOperator,
    pub value: Option<Value>,
    pub state: DocumentSummaryState,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalDocumentFilter {
    pub expression: Option<DataExpression>,
    pub operator: FilterOperator,
    pub value: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointEvidenceRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub sections: Vec<EvidenceSection>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointTelemetryTable {
    pub table: String,
    pub schema: Option<TelemetrySchema>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointEvidence {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub outputs: Vec<ExecutionFileMetadata>,
    pub artifacts: Vec<ArtifactMetadata>,
    pub record_schemas: Vec<RecordSchema>,
    pub telemetry_tables: Vec<CheckpointTelemetryTable>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadCheckpointOutputRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointFileWindow {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub content: Option<ExecutionFileWindow>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCheckpointArtifactRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub artifact_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCheckpointRecordsRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub limit: u32,
    pub after: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRecordPage {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub records: Vec<SavedRecord>,
    pub next_after: Option<String>,
    pub issues: Vec<HistoryIssue>,
    pub issues_truncated: bool,
}

// The Original Producer Epoch Qualifies The Process-Local Journal Sequence.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedRecord {
    pub epoch: String,
    pub record: Option<CapturedRecord>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCheckpointTelemetryRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
    // Original logical native stream identifier; never a Delta table name.
    #[row(description = "Original logical native stream identifier; never a Delta table name.")]
    pub table: String,
    pub limit: u32,
    pub after: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointTelemetryPage {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub table: String,
    pub rows: Vec<SavedTelemetrySample>,
    pub next_after: Option<String>,
    pub issues: Vec<HistoryIssue>,
    pub issues_truncated: bool,
}

// The Original Producer Epoch Qualifies The Process-Local Sample Ordinal.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedTelemetrySample {
    pub epoch: String,
    pub ordinal: u64,
    pub sample: Option<TelemetrySample>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointContextRequest {
    pub checkpoint: Option<ExecutionCheckpoint>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointContext {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub session_id: String,
    pub execution_id: Option<String>,
    pub execution_scope: Option<EvidenceScope>,
    pub cycle: Option<HistoricalCycleContext>,
    pub execution_documents: Vec<HistoricalDocument>,
    pub prompts: Vec<PromptInstance>,
    pub health: Vec<HealthItem>,
    pub notes: Vec<OperatorNote>,
    pub traceability: Vec<TraceabilitySnapshot>,
    pub appearance: Option<AppearanceSnapshot>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalCycleContext {
    pub cycle: Option<HistoricalCycle>,
    pub execution_order: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalCycle {
    pub cycle_id: String,
    pub metadata: Option<Metadata>,
    pub lifecycle: CycleStatus,
    pub created_at: Option<Timestamp>,
    pub started_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub cases: Vec<CaseSummary>,
    pub result: Option<ResultSummary>,
}

/* === Enums === */
// The requested order key. Ordering is always total: the key, then the
// execution ID ascending, so equal keys never permute between pages. An
// unspecified key is STARTED_AT. TITLE compares the case-folded title. An
// absent timestamp sorts after every present one in ascending order.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionOrderKey {
    Unspecified,
    StartedAt,
    FinishedAt,
    Title,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionAvailability {
    Unspecified,
    Live,
    Stored,
    Interrupted,
    Uncheckpointed,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepTermination {
    Unspecified,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSection {
    Unspecified,
    Outputs,
    Artifacts,
    RecordSchemas,
    TelemetryTables,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoricalDocumentCardBody {
    Markdown(String),
    Table(DocumentTable),
    Summary(HistoricalTelemetrySummary),
    Plot(DocumentPlotDefinition),
    Owner(DocumentOwnerReference),
}

#[harmos::catalog]
pub enum Operations {
    BrowseExecutions(BrowseExecutions),
    GetExecutionCheckpoint(GetExecutionCheckpoint),
    DeleteExecution(DeleteExecution),
    GetCheckpointDocument(GetCheckpointDocument),
    GetCheckpointEvidence(GetCheckpointEvidence),
    ReadCheckpointOutput(ReadCheckpointOutput),
    DownloadCheckpointArtifact(DownloadCheckpointArtifact),
    ListCheckpointRecords(ListCheckpointRecords),
    ListCheckpointTelemetry(ListCheckpointTelemetry),
    GetCheckpointContext(GetCheckpointContext),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for BrowseExecutions {
    const PATH: &'static str = "/dokime.v1.History/BrowseExecutions";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetExecutionCheckpoint {
    const PATH: &'static str = "/dokime.v1.History/GetExecutionCheckpoint";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for DeleteExecution {
    const PATH: &'static str = "/dokime.v1.History/DeleteExecution";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for GetCheckpointDocument {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointDocument";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetCheckpointEvidence {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointEvidence";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ReadCheckpointOutput {
    const PATH: &'static str = "/dokime.v1.History/ReadCheckpointOutput";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for DownloadCheckpointArtifact {
    const PATH: &'static str = "/dokime.v1.History/DownloadCheckpointArtifact";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCheckpointRecords {
    const PATH: &'static str = "/dokime.v1.History/ListCheckpointRecords";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCheckpointTelemetry {
    const PATH: &'static str = "/dokime.v1.History/ListCheckpointTelemetry";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetCheckpointContext {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointContext";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

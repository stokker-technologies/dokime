/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved History Discovery And Immutable Evidence Projection Contracts.

use serde::Deserialize;
use serde::Serialize;

#[allow(unused_imports)]
use crate::model::appearance::wire::*;

#[allow(unused_imports)]
use crate::model::catalog::wire::*;

#[allow(unused_imports)]
use crate::model::content::wire::*;

#[allow(unused_imports)]
use crate::model::definition::wire::*;

#[allow(unused_imports)]
use crate::model::documents::wire::*;

#[allow(unused_imports)]
use crate::model::evidence::wire::*;

#[allow(unused_imports)]
use crate::model::execution::wire::*;

#[allow(unused_imports)]
use crate::model::execution::wire_checkpoint::*;
use crate::model::identity::wire::Metadata;

#[allow(unused_imports)]
use crate::model::identity::wire::*;

#[allow(unused_imports)]
use crate::model::notes::wire::*;

#[allow(unused_imports)]
use crate::model::plots::wire::*;

#[allow(unused_imports)]
use crate::model::prompts::wire::*;

#[allow(unused_imports)]
use crate::model::telemetry::wire::*;
use crate::model::values::wire::Value;

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleBrowsePage {
    pub entries: Vec<SavedCycleSummary>,
    pub at: Option<StorageCursor>,
    pub next_after_sequence: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedCycleSummary {
    pub cycle_id: String,
    pub metadata: Option<Metadata>,
    pub phase: crate::model::execution::result::CyclePhase,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub execution_count: u64,
    pub archived: bool,
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

    #[row(description = "Absent lists only unarchived executions.")]
    pub archive: Option<crate::model::execution::wire::ArchiveScope>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub key: ExecutionOrderKey,
    pub descending: bool,
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

    #[row(
        description = "One aggregate per cycle the page's entries belong to, folded over the whole filtered selection rather than this page, so a client renders a cycle's totals without loading every page."
    )]
    pub cycles: Vec<ExecutionBrowseCycle>,
}

// Totals of one cycle's executions within a browse selection. Planned
// executions that never started (`unrun_count`) are held aside: every other
// count describes only executions that ran. `queued_count` counts executions
// admitted to run but not yet started; `running_count` counts live running
// executions. `result_status` is present only once the cycle settled: FAILED
// when any execution failed, PASSED when every execution passed.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBrowseCycle {
    pub cycle_id: String,
    pub lifecycle: ExecutionBrowseLifecycle,
    pub result_status: Option<ResultStatus>,
    pub case_count: u64,
    pub execution_count: u64,
    pub unrun_count: u64,
    pub queued_count: u64,
    pub running_count: u64,
    pub passed_count: u64,
    pub failed_count: u64,
    pub blocked_count: u64,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    // The cycle's own label, absent when its header is no longer readable.
    pub label: Option<String>,
    pub phase: Option<crate::model::execution::result::CyclePhase>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBrowseEntry {
    pub metadata: Option<HistoricalExecutionMetadata>,
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub availability: ExecutionAvailability,
    pub archived: bool,
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

    #[row(description = "Captured at admission; never reconstructed from the current suite catalog.")]
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
pub struct CheckpointFileWindow {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub content: Option<ExecutionFileWindow>,
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
    pub phase: crate::model::execution::result::CyclePhase,
    pub lifecycle: CycleStatus,
    pub created_at: Option<Timestamp>,
    pub started_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub cases: Vec<CaseSummary>,
    pub result: Option<ResultSummary>,
}

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

// Where a browsed cycle stands, by precedence: any live running execution,
// then any interrupted one, then work still pending (or nothing ran), then any
// execution without a checkpoint; otherwise every execution completed.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionBrowseLifecycle {
    Unspecified,
    Running,
    Interrupted,
    Pending,
    Uncheckpointed,
    Completed,
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

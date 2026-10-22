/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::wire::Metadata;

#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Constants === */

pub const API_PACKAGE: &str = "dokime.v1";
pub const API_VERSION: &str = "0.6.0";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One page of captured tracing events, oldest first.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceBatch {
    pub epoch: String,
    pub events: Vec<TraceEvent>,

    // Events of this scope, of any level, evicted before this stream read them.
    // Cumulative for the stream; it never rewinds.
    #[row(
        description = "Events of this scope, of any level, evicted before this stream read them. Cumulative for the stream; it never rewinds."
    )]
    pub dropped: u64,

    // The scope sequence to pass as after_sequence when resuming; it covers
    // events scanned but selected out by min_level.
    #[row(
        description = "The scope sequence to pass as after_sequence when resuming; it covers events scanned but selected out by min_level."
    )]
    pub next_sequence: u64,
}

// One tracing event. Secret-looking field names carry a redaction marker
// instead of their value, and host paths are redacted from text.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    // Position within the requested scope; strictly increasing per scope.
    #[row(description = "Position within the requested scope; strictly increasing per scope.")]
    pub sequence: u64,

    pub time: Option<Timestamp>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,

    // The plugin that emitted or forwarded the line, when known.
    #[row(description = "The plugin that emitted or forwarded the line, when known.")]
    pub plugin_id: Option<String>,

    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub fields: Vec<LogFrameFieldsEntry>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendIdentity {
    // Stable identity of this backend installation, not its network address.
    #[row(description = "Stable identity of this backend installation, not its network address.")]
    pub backend_id: String,

    // Unique for every runtime boot; invalidates process-local cursors.
    #[row(description = "Unique for every runtime boot; invalidates process-local cursors.")]
    pub epoch: String,

    pub api: Option<ApiIdentity>,
    pub product_version: String,
    pub source_revision: Option<String>,
    pub sdk_revision: Option<String>,
    pub started_at: Option<Timestamp>,
    pub display_name: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiIdentity {
    // Fully qualified protobuf package, currently dokime.v1.
    #[row(description = "Fully qualified protobuf package, currently dokime.v1.")]
    pub package: String,

    // Contract version independent of product and SDK version.
    #[row(description = "Contract version independent of product and SDK version.")]
    pub version: String,

    // SHA-256 of the released descriptor set, including imports.
    #[row(description = "SHA-256 of the released descriptor set, including imports.")]
    pub descriptor_sha256: Vec<u8>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend_id: String,
    pub epoch: String,
    pub readiness: Readiness,
    pub observed_at: Option<Timestamp>,

    // Human-readable explanation; callers branch on readiness, not this text.
    #[row(description = "Human-readable explanation; callers branch on readiness, not this text.")]
    pub message: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendAvailability {
    pub backend_id: String,
    pub epoch: String,

    // Methods both implemented and available to the authenticated caller.
    // Full names, e.g. /dokime.v1.Backend/GetIdentity.
    #[row(
        description = "Methods both implemented and available to the authenticated caller. Full names, e.g. /dokime.v1.Backend/GetIdentity."
    )]
    pub methods: Vec<String>,

    pub limits: Option<BackendLimits>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendLimits {
    // Positive limits enforced by this backend, excluding transport framing.
    #[row(description = "Positive limits enforced by this backend, excluding transport framing.")]
    pub max_request_bytes: u64,

    pub max_response_bytes: u64,

    // Optional until corresponding streaming/query services are implemented.
    #[row(description = "Optional until corresponding streaming/query services are implemented.")]
    pub max_content_bytes: Option<u64>,

    pub max_evidence_bytes: Option<u64>,

    pub max_chunk_bytes: Option<u32>,
    pub max_page_items: Option<u32>,
    pub max_stream_buffer_items: Option<u32>,
}

// Native diagnostics intentionally reports backend and runtime facts. It does
// not mirror HTTP listener policy such as origins, heartbeat intervals, or
// bind addresses.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    pub server: Option<BackendDiagnostics>,
    pub runtime: Option<RuntimeDiagnostics>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendDiagnostics {
    pub version: String,
    pub process: Option<ProcessDiagnostics>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessDiagnostics {
    pub pid: u32,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub virtual_memory_bytes: Option<u64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeDiagnostics {
    pub captured_at: Option<Timestamp>,
    pub session_id: String,
    pub identity: Option<RuntimeIdentity>,
    pub revision: u64,
    pub shutting_down: bool,
    pub storage_root: String,
    pub hub_metadata: Option<Metadata>,
    pub startup_doctor: Option<StartupDoctorSnapshot>,
    pub path_conventions: Vec<DiagnosticPathConvention>,
    pub counts: Option<DiagnosticsCounts>,
    pub installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeIdentity {
    pub node: Option<RuntimeNodeIdentity>,
    pub actor: Option<RuntimeActorIdentity>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeNodeIdentity {
    pub node_id: String,
    pub hostname_label: Option<String>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeActorIdentity {
    pub actor_id: String,
    pub display_name: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StartupDoctorSnapshot {
    pub checked_at: Option<Timestamp>,
    pub status: DiagnosticCheckStatus,
    pub checks: Vec<StartupDoctorCheck>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StartupDoctorCheck {
    pub key: StartupDoctorCheckKey,
    pub status: DiagnosticCheckStatus,
    pub message: String,
    pub detail: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticPathConvention {
    pub key: DiagnosticPathKey,
    pub scope: DiagnosticPathScope,
    pub format: DiagnosticFileFormat,
    pub relative_path: String,
    pub absolute_path: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsCounts {
    pub test_suites: u64,
    pub cycles: u64,
    pub installed_plugin_errors: u64,
    pub state_cycles: u64,
    pub state_executions_total: u64,
    pub state_record_count: u64,
    pub telemetry_cursor_entries: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledPluginErrorRecord {
    pub plugin_id: String,
    pub contribution_id: Option<String>,
    pub phase: InstalledPluginPhase,
    pub level: Severity,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<ErrorCause>,
    pub stacktrace_attachment: Option<String>,
    pub trace_id: Option<String>,
    pub scope: Option<EvidenceScope>,
    pub session_id: Option<String>,
    pub test_cycle_id: Option<String>,
    pub test_execution_id: Option<String>,
    pub recorded_at: Option<Timestamp>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorCause {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

// One diagnostics archive, delivered as a header, contiguous chunks, and a
// completion. Accept the bytes only after the completion frame and a
// successful final gRPC status.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleFrame {
    pub payload: Option<DiagnosticsBundleFramePayload>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleHeader {
    // Suggested save name, dokime-diagnostics-<utc timestamp>.zip. It is a name,
    // not a backend path: nothing is written to the storage root.
    #[row(
        description = "Suggested save name, dokime-diagnostics-<utc timestamp>.zip. It is a name, not a backend path: nothing is written to the storage root."
    )]
    pub filename: String,

    pub content_type: String,
    pub generated_at: Option<Timestamp>,

    // Present whenever the archive is complete before the first chunk, which is
    // how this backend builds it.
    #[row(description = "Present whenever the archive is complete before the first chunk, which is how this backend builds it.")]
    pub total_size_bytes: Option<u64>,

    // The same inventory the archive's manifest.json carries.
    #[row(description = "The same inventory the archive's manifest.json carries.")]
    pub entries: Vec<DiagnosticsBundleEntry>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleEntry {
    // Archive-relative path, never a backend filesystem path.
    #[row(description = "Archive-relative path, never a backend filesystem path.")]
    pub path: String,

    pub size_bytes: u64,

    // False when the source file does not exist; the entry is still listed so an
    // absent convention is distinguishable from an unattempted one.
    #[row(
        description = "False when the source file does not exist; the entry is still listed so an absent convention is distinguishable from an unattempted one."
    )]
    pub present: bool,

    pub note: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleCompletion {
    pub size_bytes: u64,

    // SHA-256 of exactly the bytes delivered by preceding chunks.
    #[row(description = "SHA-256 of exactly the bytes delivered by preceding chunks.")]
    pub sha256: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LogFrameFieldsEntry {
    pub key: String,
    pub value: String,
}

// One observed hub-log record. Secret-looking field names carry a redaction
// marker instead of their value.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LogFrame {
    pub time: Option<Timestamp>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: Vec<LogFrameFieldsEntry>,

    // Matching records discarded ahead of this one because the client fell
    // behind. Cumulative for the stream; it never rewinds.
    #[row(
        description = "Matching records discarded ahead of this one because the client fell behind. Cumulative for the stream; it never rewinds."
    )]
    pub dropped: u64,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Readiness {
    Unspecified,
    Starting,
    Ready,
    Degraded,
    Stopping,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCheckStatus {
    Unspecified,
    Ok,
    Warning,
    Failed,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupDoctorCheckKey {
    Unspecified,
    StorageRoot,
    ShutdownState,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticPathKey {
    Unspecified,
    HubLog,
    StartupDoctor,
    DiagnosticsSnapshot,
    CollectBundle,
    CycleLogPattern,
    ExecutionLogPattern,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticPathScope {
    Unspecified,
    Root,
    CyclePattern,
    ExecutionPattern,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticFileFormat {
    Unspecified,
    Json,
    Jsonl,
    Zip,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstalledPluginPhase {
    Unspecified,
    Entrypoint,
    Action,
    Sidecar,
    Provider,
    Initialize,
    Run,
    Finalize,
    Interrupt,
}

// Hub log severity. Callers branch on this, never on the log text.
#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Unspecified,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Which captured events a trace watch reads. Cycle and Execution carry the
// canonical UUID of that cycle or execution.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceScope {
    Instance,
    Cycle(String),
    Execution(String),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticsBundleFramePayload {
    Header(DiagnosticsBundleHeader),
    Chunk(DiagnosticsBundleChunk),
    Complete(DiagnosticsBundleCompletion),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn compatible_api(package: &str, version: &str) -> bool {
    fn release(version: &str) -> Option<(&str, &str)> {
        let parts: Vec<_> = version.split('.').collect();
        if parts.len() != 3
            || parts
                .iter()
                .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) || (part.len() > 1 && part.starts_with('0')))
        {
            return None;
        }

        Some((parts[0], parts[1]))
    }

    package == API_PACKAGE && release(version).is_some_and(|version| Some(version) == release(API_VERSION))
}

/* === Private Functions === */

/* === Tests === */

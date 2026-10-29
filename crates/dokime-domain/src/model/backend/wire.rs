/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

#[allow(unused_imports)]
use crate::model::identity::wire::*;

pub const API_PACKAGE: &str = "dokime.v1";
pub const API_VERSION: &str = "0.6.0";

// One page of captured tracing events, oldest first.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceBatch {
    pub epoch: String,
    pub events: Vec<TraceEvent>,

    #[row(
        description = "Events of this scope, of any level, evicted before this stream read them. Cumulative for the stream; it never rewinds."
    )]
    pub dropped: u64,

    #[row(
        description = "The scope sequence to pass as after_sequence when resuming; it covers events scanned but selected out by min_level."
    )]
    pub next_sequence: u64,
}

// One tracing event. Secret-looking field names carry a redaction marker
// instead of their value, and host paths are redacted from text.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    #[row(description = "Position within the requested scope; strictly increasing per scope.")]
    pub sequence: u64,

    pub time: Option<Timestamp>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,

    #[row(description = "The plugin that emitted or forwarded the line, when known.")]
    pub plugin_id: Option<String>,

    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub fields: Vec<LogFrameFieldsEntry>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendIdentity {
    #[row(description = "Stable identity of this backend installation, not its network address.")]
    pub backend_id: String,

    #[row(description = "Unique for every runtime boot; invalidates process-local cursors.")]
    pub epoch: String,

    pub api: Option<ApiIdentity>,
    pub product_version: String,
    pub source_revision: Option<String>,
    pub sdk_revision: Option<String>,
    pub started_at: Option<Timestamp>,
    pub display_name: String,
    #[row(description = "Authoritative Rust target of the backend that admits plugin sidecars.")]
    pub target: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiIdentity {
    #[row(description = "Fully qualified protobuf package, currently dokime.v1.")]
    pub package: String,

    #[row(description = "Contract version independent of product and SDK version.")]
    pub version: String,

    #[row(description = "SHA-256 of the released descriptor set, including imports.")]
    pub descriptor_sha256: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend_id: String,
    pub epoch: String,
    pub readiness: Readiness,
    pub observed_at: Option<Timestamp>,

    #[row(description = "Human-readable explanation; callers branch on readiness, not this text.")]
    pub message: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendAvailability {
    pub backend_id: String,
    pub epoch: String,

    #[row(
        description = "Methods both implemented and available to the authenticated caller. Full names, e.g. /dokime.v1.Backend/GetIdentity."
    )]
    pub methods: Vec<String>,

    pub limits: Option<BackendLimits>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendLimits {
    #[row(description = "Positive limits enforced by this backend, excluding transport framing.")]
    pub max_request_bytes: u64,

    pub max_response_bytes: u64,

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
// One diagnostics archive, delivered as a header, contiguous chunks, and a
// completion. Accept the bytes only after the completion frame and a
// successful final gRPC status.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleFrame {
    pub payload: Option<DiagnosticsBundleFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleHeader {
    #[row(
        description = "Suggested save name, dokime-diagnostics-<utc timestamp>.zip. It is a name, not a backend path: nothing is written to the storage root."
    )]
    pub filename: String,

    pub content_type: String,
    pub generated_at: Option<Timestamp>,

    #[row(description = "Present whenever the archive is complete before the first chunk, which is how this backend builds it.")]
    pub total_size_bytes: Option<u64>,

    #[row(description = "The same inventory the archive's manifest.json carries.")]
    pub entries: Vec<DiagnosticsBundleEntry>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleEntry {
    #[row(description = "Archive-relative path, never a backend filesystem path.")]
    pub path: String,

    pub size_bytes: u64,

    #[row(
        description = "False when the source file does not exist; the entry is still listed so an absent convention is distinguishable from an unattempted one."
    )]
    pub present: bool,

    pub note: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsBundleCompletion {
    pub size_bytes: u64,

    #[row(description = "SHA-256 of exactly the bytes delivered by preceding chunks.")]
    pub sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LogFrameFieldsEntry {
    pub key: String,
    pub value: String,
}

// One observed hub-log record. Secret-looking field names carry a redaction
// marker instead of their value. A frame with an unspecified level carries no
// record: it is a quiet watch's keepalive.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LogFrame {
    pub time: Option<Timestamp>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: Vec<LogFrameFieldsEntry>,

    #[row(
        description = "Matching records discarded ahead of this one because the client fell behind. Cumulative for the stream; it never rewinds."
    )]
    pub dropped: u64,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Readiness {
    Unspecified,
    Starting,
    Ready,
    Degraded,
    Stopping,
}

// Hub log severity. Callers branch on this, never on the log text.
#[harmos::message]
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
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceScope {
    Instance,
    Cycle(String),
    Execution(String),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticsBundleFramePayload {
    Header(DiagnosticsBundleHeader),
    Chunk(DiagnosticsBundleChunk),
    Complete(DiagnosticsBundleCompletion),
}

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

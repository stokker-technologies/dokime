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
#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::execution::wire_cursor::Cursor;

#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedRecordSelection {
    // Conjunction of exact supplied fields. Missing record fields do not match.
    #[row(description = "Conjunction of exact supplied fields. Missing record fields do not match.")]
    pub scope: Option<EvidenceScope>,

    pub record_types: Vec<String>,
    pub sources: Vec<String>,
    pub severities: Vec<Severity>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordPage {
    pub records: Vec<CapturedRecord>,

    // Scanned journal position, including excluded records and empty ranges.
    #[row(description = "Scanned journal position, including excluded records and empty ranges.")]
    pub scanned: Option<Cursor>,

    pub through: Option<Cursor>,
    pub replay_complete: bool,

    // This process retains all captured facts. Zero is currently the only value.
    #[row(description = "This process retains all captured facts. Zero is currently the only value.")]
    pub retained_after: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedRecord {
    pub record_id: String,
    pub sequence: u64,

    // Authored UTC occurrence time, not capture or commit time.
    #[row(description = "Authored UTC occurrence time, not capture or commit time.")]
    pub timestamp: Option<Timestamp>,

    pub record_type: String,
    pub schema_version: u32,
    pub source: String,
    pub severity: Severity,
    pub message: Option<String>,
    pub error: Option<CapturedRecordError>,
    pub details: Option<Value>,
    pub scope: Option<EvidenceScope>,
    pub related_sample_id: Option<String>,
    pub related_stream_id: Option<String>,
    pub related_artifact_id: Option<String>,
    pub received_at: Option<Timestamp>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedRecordError {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<CapturedErrorCause>,
    pub stacktrace_attachment: Option<String>,
    pub trace_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedErrorCause {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

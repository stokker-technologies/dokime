/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod list_samples;

/* === Re-Exports === */
pub use list_samples::ListSamples;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::common::*;
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTelemetrySamplesRequestAfterStreamCursorsEntry {
    pub key: String,
    pub value: u64,
}

// A zero limit means the documented default of 500. Values above 5,000 are
// rejected; the encoded reply must also fit the negotiated gRPC message limit.
// No response is silently truncated: lower the limit and retry after a
// ResourceExhausted response.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTelemetrySamplesRequest {
    pub streams: Vec<String>,
    pub limit: u32,
    pub schema_versions: Vec<u32>,
    pub sources: Vec<String>,
    pub signals: Vec<String>,
    pub scope: Option<EvidenceScope>,
    pub after_stream_cursors: Vec<ListTelemetrySamplesRequestAfterStreamCursorsEntry>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    // Selects an explicit tail window of the filtered selection. A latest
    // request cannot carry after_stream_cursors; use latest=false for forward
    // continuation through the full selection.
    #[row(
        description = "Selects an explicit tail window of the filtered selection. A latest request cannot carry after_stream_cursors; use latest=false for forward continuation through the full selection."
    )]
    pub latest: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySamplesReplyThroughStreamCursorsEntry {
    pub key: String,
    pub value: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySamplesReply {
    pub samples: Vec<TelemetrySample>,
    // The greatest sequence returned for each stream. For latest=false, a
    // caller can use these positions as the next after_stream_cursors value.
    // Latest-window replies leave this map empty because older samples are
    // intentionally outside that request's selection.
    #[row(
        description = "The greatest sequence returned for each stream. For latest=false, a caller can use these positions as the next after_stream_cursors value. Latest-window replies leave this map empty because older samples are intentionally outside that request's selection."
    )]
    pub through_stream_cursors: Vec<TelemetrySamplesReplyThroughStreamCursorsEntry>,
    // For latest=false, false means the service reached the limit and the full
    // filtered selection may contain more samples; continue with the returned
    // cursors. For latest=true, the selected tail window is complete even when
    // older samples exist outside that explicit window.
    #[row(
        description = "For latest=false, false means the service reached the limit and the full filtered selection may contain more samples; continue with the returned cursors. For latest=true, the selected tail window is complete even when older samples exist outside that explicit window."
    )]
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub sample_id: String,
    pub stream: String,
    pub schema_version: u32,
    pub sequence: u64,
    pub timestamp: Option<Timestamp>,
    pub received_at: Option<Timestamp>,
    pub source: String,
    pub scope: Option<EvidenceScope>,
    pub values: Option<Value>,
    pub summary: Option<Value>,
    pub metadata: Option<Value>,
}

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListSamples(ListSamples),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/ListSamples";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
    // For latest=false, false means the page reached its limit or the message
    // budget and the full filtered selection may contain more samples; continue
    // with the returned cursors. For latest=true, the selected tail window is
    // complete even when older samples exist outside that explicit window, and
    // false means the budget clamped the window to its newest samples.
    #[row(
        description = "For latest=false, false means the page reached its limit or the message budget and the full filtered selection may contain more samples; continue with the returned cursors. For latest=true, the selected tail window is complete even when older samples exist outside that explicit window, and false means the budget clamped the window to its newest samples."
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

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

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<ListSamples>], response = TelemetrySamplesReply, id = "dokime.telemetry.list_samples",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Telemetry", method = "ListSamples"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListSamples {
    pub streams: Vec<String>,

    #[row(
        description = "Most samples one page returns. Zero uses 500; larger values are clamped to 5,000, and a page is further clamped to the samples that fit the negotiated message budget. A clamped page reports complete=false: for latest=false it keeps the oldest samples and its cursors continue after them; for latest=true it keeps the newest. No request is refused for asking too much."
    )]
    pub limit: u32,

    pub schema_versions: Vec<u32>,
    pub sources: Vec<String>,
    pub signals: Vec<String>,
    pub scope: Option<EvidenceScope>,
    pub after_stream_cursors: Vec<ListTelemetrySamplesRequestAfterStreamCursorsEntry>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,

    #[row(
        description = "Selects an explicit tail window of the filtered selection. A latest request cannot carry after_stream_cursors; use latest=false for forward continuation through the full selection."
    )]
    pub latest: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListSamples);
}

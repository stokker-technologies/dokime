/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

use serde::Deserialize;
use serde::Serialize;

use crate::model::identity::wire::Metadata;

#[allow(unused_imports)]
use crate::model::identity::wire::*;

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializedReport {
    pub report: harmos_domain::model::storage::FileReference,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub test_cycle_id: String,

    // Set when the report is one execution's rather than its cycle's.
    #[row(description = "Set when the report is one execution's rather than its cycle's.")]
    pub test_execution_id: Option<String>,

    pub identifier: String,
    pub metadata: Option<Metadata>,
    pub generated_at: Option<Timestamp>,
    pub origin: ReportOrigin,
    pub warnings: Vec<String>,
}

// Accept the download only after completion and a successful final status.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlFrame {
    pub payload: Option<ReportHtmlFramePayload>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlCompletion {
    pub size_bytes: u64,

    // SHA-256 of exactly the bytes delivered by preceding chunks.
    #[row(description = "SHA-256 of exactly the bytes delivered by preceding chunks.")]
    pub sha256: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportSubject {
    Execution(String),
    Cycle(String),
}

// Where the delivered report came from.
#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportOrigin {
    // The report finalization sealed, read back unchanged.
    Sealed,
    // The subject is not sealed: rendered from its current evidence.
    Current,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportHtmlFramePayload {
    Report(Box<ReportMetadata>),
    Chunk(ReportHtmlChunk),
    Complete(ReportHtmlCompletion),
}

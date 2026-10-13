/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod download_cycle_report;
mod render_checkpoint_report;

/* === Re-Exports === */
pub use download_cycle_report::DownloadCycleReport;
pub use render_checkpoint_report::RenderCheckpointReport;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::checkpoint::*;
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
use super::{Access, Endpoint, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCycleReportRequest {
    // Canonical UUID of a saved cycle.
    #[row(description = "Canonical UUID of a saved cycle.")]
    pub cycle_id: String,
    pub identifier: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderCheckpointReportRequest {
    pub context: Option<ExecutionCheckpoint>,
    pub checkpoints: Vec<ExecutionCheckpoint>,
    pub identifier: String,
}

// Legacy preview response shape. Native storage does not implement rerendering.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderedReport {
    pub identifier: String,
    pub metadata: Option<Metadata>,
    pub generated_at: Option<Timestamp>,
    pub warnings: Vec<String>,
    pub html: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub test_cycle_id: String,
    pub identifier: String,
    pub metadata: Option<Metadata>,
    pub generated_at: Option<Timestamp>,
    pub warnings: Vec<String>,
}

// Accept the download only after completion and a successful final status.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlFrame {
    pub payload: Option<ReportHtmlFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportHtmlCompletion {
    pub size_bytes: u64,
    // SHA-256 of exactly the bytes delivered by preceding chunks.
    #[row(description = "SHA-256 of exactly the bytes delivered by preceding chunks.")]
    pub sha256: String,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportHtmlFramePayload {
    Report(ReportMetadata),
    Chunk(ReportHtmlChunk),
    Complete(ReportHtmlCompletion),
}

#[harmos::catalog]
pub enum Operations {
    DownloadCycleReport(DownloadCycleReport),
    RenderCheckpointReport(RenderCheckpointReport),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for DownloadCycleReport {
    const PATH: &'static str = "/dokime.v1.Reports/DownloadCycleReport";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for RenderCheckpointReport {
    const PATH: &'static str = "/dokime.v1.Reports/RenderCheckpointReport";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

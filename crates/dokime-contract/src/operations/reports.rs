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
mod download_execution_report;
mod materialize_report;

/* === Re-Exports === */
pub use download_cycle_report::DownloadCycleReport;
pub use download_execution_report::DownloadExecutionReport;
pub use materialize_report::MaterializeReport;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
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
pub struct MaterializeReportRequest {
    pub subject: ReportSubject,
    pub at: harmos::storage::StorageCursor,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializedReport {
    pub report: harmos::storage::FileReference,
    pub materialized_input: Option<harmos::storage::FileReference>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportSubject {
    Execution(String),
    Cycle(String),
}

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
pub struct DownloadExecutionReportRequest {
    // Canonical UUID of one execution, running or settled.
    #[row(description = "Canonical UUID of one execution, running or settled.")]
    pub execution_id: String,
}

#[harmos::message]
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
// Where the delivered report came from.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportOrigin {
    // The report finalization sealed, read back unchanged.
    Sealed,
    // The subject is not sealed: rendered from its current evidence.
    Current,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportHtmlFramePayload {
    Report(Box<ReportMetadata>),
    Chunk(ReportHtmlChunk),
    Complete(ReportHtmlCompletion),
}

#[harmos::catalog]
pub enum Operations {
    MaterializeReport(MaterializeReport),
    DownloadCycleReport(DownloadCycleReport),
    DownloadExecutionReport(DownloadExecutionReport),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for MaterializeReport {
    const PATH: &'static str = "/dokime.v1.Reports/MaterializeReport";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for DownloadCycleReport {
    const PATH: &'static str = "/dokime.v1.Reports/DownloadCycleReport";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for DownloadExecutionReport {
    const PATH: &'static str = "/dokime.v1.Reports/DownloadExecutionReport";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

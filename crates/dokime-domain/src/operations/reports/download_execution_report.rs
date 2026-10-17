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

// Local Imports
use super::{DownloadExecutionReportRequest, ReportHtmlFrame, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Streams one execution's report: the one its finalization sealed, or one
// generated from its current or stored evidence when there is none to read.
#[harmos::operation(request = DownloadExecutionReportRequest, response = (), output = ReportHtmlFrame, id = "dokime.reports.download_execution_report", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Reports", method = "DownloadExecutionReport"))]
pub struct DownloadExecutionReport;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

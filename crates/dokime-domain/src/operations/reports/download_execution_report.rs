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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Streams one execution's report: the one its finalization sealed, or one
// generated from its current or stored evidence when there is none to read.
#[harmos::message]
#[harmos::operation(response = (), output = ReportHtmlFrame, id = "dokime.reports.download_execution_report", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Reports", method = "DownloadExecutionReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadExecutionReport {
    // Canonical UUID of one execution, running or settled.
    #[row(description = "Canonical UUID of one execution, running or settled.")]
    pub execution_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

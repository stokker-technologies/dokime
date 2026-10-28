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

use super::*;

// Streams one execution's report: the one its finalization sealed, or one
// generated from its current or stored evidence when there is none to read.
#[harmos::operation(resources = [crate::hosting::Host<DownloadExecutionReport>], response = (), output = ReportHtmlFrame, id = "dokime.reports.download_execution_report",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Reports", method = "DownloadExecutionReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadExecutionReport {
    // Canonical UUID of one execution, running or settled.
    #[row(description = "Canonical UUID of one execution, running or settled.")]
    pub execution_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadExecutionReport);
}

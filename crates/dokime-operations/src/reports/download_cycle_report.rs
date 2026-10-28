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

#[harmos::operation(resources = [crate::hosting::Host<DownloadCycleReport>], response = (), output = ReportHtmlFrame, id = "dokime.reports.download_cycle_report",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Reports", method = "DownloadCycleReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCycleReport {
    // Canonical UUID of a saved cycle.
    #[row(description = "Canonical UUID of a saved cycle.")]
    pub cycle_id: String,

    pub identifier: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadCycleReport);
}

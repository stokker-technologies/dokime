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
#[harmos::message]
#[harmos::operation(response = (), output = ReportHtmlFrame, id = "dokime.reports.download_cycle_report", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Reports", method = "DownloadCycleReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCycleReport {
    // Canonical UUID of a saved cycle.
    #[row(description = "Canonical UUID of a saved cycle.")]
    pub cycle_id: String,
    pub identifier: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

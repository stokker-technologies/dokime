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

// Local Imports
use super::TRANSFER;

use dokime_domain::model::reports::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    MaterializeReport(MaterializeReport),
    DownloadCycleReport(DownloadCycleReport),
    DownloadExecutionReport(DownloadExecutionReport),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

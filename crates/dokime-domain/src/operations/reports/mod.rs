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
use super::Access;
use super::Endpoint;
use super::TRANSFER;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::reports::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
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

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod download_cycle_report;
mod download_execution_report;
mod materialize_report;

pub use download_cycle_report::DownloadCycleReport;
pub use download_execution_report::DownloadExecutionReport;
pub use materialize_report::MaterializeReport;

use super::Endpoint;
use super::TRANSFER;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::reports::wire::*;

#[harmos::catalog]
pub enum Operations {
    MaterializeReport(MaterializeReport),
    DownloadCycleReport(DownloadCycleReport),
    DownloadExecutionReport(DownloadExecutionReport),
}

impl Endpoint for MaterializeReport {
    const PATH: &'static str = "/dokime.v1.Reports/MaterializeReport";
}

impl Endpoint for DownloadCycleReport {
    const PATH: &'static str = "/dokime.v1.Reports/DownloadCycleReport";
}

impl Endpoint for DownloadExecutionReport {
    const PATH: &'static str = "/dokime.v1.Reports/DownloadExecutionReport";
}

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
use super::{MaterializeReportRequest, MaterializedReport, TRANSFER};
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
#[harmos::operation(request = MaterializeReportRequest, response = MaterializedReport,
    id = "dokime.reports.materialize_report", version = 1, policy = TRANSFER,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Reports", method = "MaterializeReport"))]
pub struct MaterializeReport;
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
/* === Private Functions === */
/* === Tests === */

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
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use super::*;

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
#[harmos::message]
#[harmos::operation(response = MaterializedReport,
    id = "dokime.reports.materialize_report", version = 1, policy = TRANSFER,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Reports", method = "MaterializeReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializeReport {
    pub subject: ReportSubject,
    pub at: harmos::storage::StorageCursor,
}
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
/* === Private Functions === */
/* === Tests === */

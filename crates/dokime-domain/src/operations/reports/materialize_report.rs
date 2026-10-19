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

#[harmos_domain::message]
#[harmos_domain::operation(response = MaterializedReport,
    id = "dokime.reports.materialize_report", version = 1, policy = TRANSFER,
    visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Reports", method = "MaterializeReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializeReport {
    pub subject: ReportSubject,
    pub at: harmos_domain::model::storage::StorageCursor,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

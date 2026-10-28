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

#[harmos::operation(resources = [crate::hosting::Host<MaterializeReport>], response = MaterializedReport,
    id = "dokime.reports.materialize_report", policy = TRANSFER,
    visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Reports", method = "MaterializeReport"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializeReport {
    pub subject: ReportSubject,
    pub at: harmos_domain::model::storage::StorageCursor,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(MaterializeReport);
}

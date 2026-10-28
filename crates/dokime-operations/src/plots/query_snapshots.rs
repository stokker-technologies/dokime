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

#[harmos::operation(resources = [crate::hosting::Host<QuerySnapshots>], response = PlotSnapshotsReply, id = "dokime.plots.query_snapshots",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plots", method = "QuerySnapshots"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QuerySnapshots {
    pub requests: Vec<PlotRequest>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(QuerySnapshots);
}

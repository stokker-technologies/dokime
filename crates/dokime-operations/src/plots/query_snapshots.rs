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

// The native bridge and server reject a request with more than 32 plots. The
// reply contains repeated envelopes in one bounded message; callers must
// lower max_points or split a batch on exhaustion.
#[harmos::operation(resources = [crate::hosting::Host<QuerySnapshots>], response = PlotSnapshotsReply, id = "dokime.plots.query_snapshots",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
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

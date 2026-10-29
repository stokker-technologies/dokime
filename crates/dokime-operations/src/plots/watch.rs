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

// Watch is server-streaming. To change the request set, cancel this stream and
// open another one. Requests without a cursor receive an initial Snapshot
// before the watch begins; later frames are coalesced by the server at 100 ms
// only after a runtime state or telemetry watch advances.
#[harmos::operation(resources = [crate::hosting::Host<Watch>], response = (), output = PlotStreamFrame, id = "dokime.plots.watch",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plots", method = "Watch"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Watch {
    pub requests: Vec<PlotRequest>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(Watch);
}

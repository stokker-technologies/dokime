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

#[harmos::operation(resources = [crate::hosting::Host<Watch>], response = (), output = PlotStreamFrame, id = "dokime.plots.watch",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
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

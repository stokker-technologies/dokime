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

// Analysis IDs Are Open; Each Contribution Declares Its Parameters And Outputs.
#[harmos::operation(resources = [crate::hosting::Host<ListAnalyses>], response = ListAnalysesReply, id = "dokime.plots.list_analyses",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plots", method = "ListAnalyses"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAnalyses {
    pub limit: u32,
    pub after_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListAnalyses);
}

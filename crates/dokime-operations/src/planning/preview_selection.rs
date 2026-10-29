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

#[harmos::operation(resources = [crate::hosting::Host<PreviewSelection>], response = PreviewSelectionReply, id = "dokime.planning.preview_selection",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Planning", method = "PreviewSelection"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewSelection {
    pub plan: Option<SelectionPlan>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(PreviewSelection);
}

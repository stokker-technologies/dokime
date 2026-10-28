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

#[harmos::operation(resources = [crate::hosting::Host<ListArtifacts>], response = ListArtifactsReply, id = "dokime.content.list_artifacts",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ListArtifacts"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListArtifacts {
    pub limit: u32,
    pub after_artifact_id: String,
    pub cursor: Option<ArtifactCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListArtifacts);
}

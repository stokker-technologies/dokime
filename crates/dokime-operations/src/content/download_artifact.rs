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

#[harmos::operation(resources = [crate::hosting::Host<DownloadArtifact>], response = (), output = ArtifactFrame, id = "dokime.content.download_artifact",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadArtifact"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub artifact_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadArtifact);
}

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

#[harmos::operation(resources = [crate::hosting::Host<DownloadCheckpointArtifact>], response = (), output = ArtifactFrame, id = "dokime.history.download_checkpoint_artifact",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "DownloadCheckpointArtifact"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCheckpointArtifact {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub artifact_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadCheckpointArtifact);
}

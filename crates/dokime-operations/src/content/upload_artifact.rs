/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

// Uploads one immutable artifact into the runtime's active execution. The first frame is a header and the remaining frames are contiguous chunks; end of stream commits the runtime upload.
#[harmos::operation(resources = [crate::hosting::Host<UploadArtifact>], response = ArtifactMetadata, input = UploadArtifactFrame, id = "dokime.content.upload_artifact",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "UploadArtifact"))]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UploadArtifact {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UploadArtifact);
}

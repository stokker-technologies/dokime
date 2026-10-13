/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{ArtifactMetadata, TRANSFER, UploadArtifactFrame};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Uploads one immutable artifact into the runtime's active execution. The first frame is a header and the remaining frames are contiguous chunks; end of stream commits the runtime upload.
#[harmos::operation(request = (), response = ArtifactMetadata, input = UploadArtifactFrame, id = "dokime.content.upload_artifact", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "UploadArtifact"))]
pub struct UploadArtifact;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

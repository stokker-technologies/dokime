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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::operation(response = (), output = ArtifactFrame, id = "dokime.content.download_artifact",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadArtifact"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub artifact_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

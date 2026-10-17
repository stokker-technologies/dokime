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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = (), output = ArtifactFrame, id = "dokime.history.download_checkpoint_artifact", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "DownloadCheckpointArtifact"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadCheckpointArtifact {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub artifact_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

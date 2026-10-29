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

#[harmos::operation(resources = [crate::hosting::Host<GetCheckpointDocument>], response = CheckpointDocument, id = "dokime.history.get_checkpoint_document",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetCheckpointDocument"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointDocument {
    pub checkpoint: Option<ExecutionCheckpoint>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCheckpointDocument);
}

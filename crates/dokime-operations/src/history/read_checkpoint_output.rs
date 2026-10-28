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

#[harmos::operation(resources = [crate::hosting::Host<ReadCheckpointOutput>], response = CheckpointFileWindow, id = "dokime.history.read_checkpoint_output",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "ReadCheckpointOutput"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadCheckpointOutput {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ReadCheckpointOutput);
}

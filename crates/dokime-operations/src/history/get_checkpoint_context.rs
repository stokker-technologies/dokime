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

#[harmos::operation(resources = [crate::hosting::Host<GetCheckpointContext>], response = CheckpointContext, id = "dokime.history.get_checkpoint_context",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetCheckpointContext"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointContext {
    pub checkpoint: Option<ExecutionCheckpoint>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCheckpointContext);
}

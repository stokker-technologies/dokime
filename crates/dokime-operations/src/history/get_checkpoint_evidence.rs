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

#[harmos::operation(resources = [crate::hosting::Host<GetCheckpointEvidence>], response = CheckpointEvidence, id = "dokime.history.get_checkpoint_evidence",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetCheckpointEvidence"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointEvidence {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub sections: Vec<EvidenceSection>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCheckpointEvidence);
}

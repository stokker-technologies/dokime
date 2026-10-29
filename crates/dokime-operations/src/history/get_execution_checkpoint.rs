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

// Bounded inspection of one existing saved seal; never flushes or publishes. NOT_FOUND means no saved seal is available, including an unsealed row.
#[harmos::operation(resources = [crate::hosting::Host<GetExecutionCheckpoint>], response = ExecutionCheckpoint, id = "dokime.history.get_execution_checkpoint",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetExecutionCheckpoint"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetExecutionCheckpoint {
    pub execution_identifier: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetExecutionCheckpoint);
}

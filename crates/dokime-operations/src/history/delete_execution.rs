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

#[harmos::operation(resources = [crate::hosting::Host<DeleteExecution>], response = DeleteExecutionResponse, id = "dokime.history.delete_execution",
        policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "DeleteExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteExecution {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub request_identifier: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DeleteExecution);
}

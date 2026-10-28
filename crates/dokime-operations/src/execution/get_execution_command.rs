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

#[harmos::operation(resources = [crate::hosting::Host<GetExecutionCommand>], response = ExecutionCommandReceipt, id = "dokime.execution.get_execution_command",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "GetExecutionCommand"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetExecutionCommand {
    pub request_id: String,
    pub command_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetExecutionCommand);
}

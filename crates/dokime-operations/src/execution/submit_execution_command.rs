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

#[harmos::operation(resources = [crate::hosting::Host<SubmitExecutionCommand>], response = ExecutionCommandReceipt, id = "dokime.execution.submit_execution_command",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SubmitExecutionCommand"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitExecutionCommand {
    pub request_id: String,
    pub command_id: String,
    pub execution_id: String,
    pub action: ExecutionCommandAction,
    pub step_id: Option<String>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(SubmitExecutionCommand);
}

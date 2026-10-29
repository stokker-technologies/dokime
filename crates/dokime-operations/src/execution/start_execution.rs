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

// Admit one existing, unstarted execution. Never reopens a finished attempt.
#[harmos::operation(resources = [crate::hosting::Host<StartExecution>], response = RunReply, id = "dokime.execution.start_execution",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "StartExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StartExecution {
    pub execution_id: String,
    pub mode: ExecutionMode,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(StartExecution);
}

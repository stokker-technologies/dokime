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

// Controls require that execution's active execution-addressed request.
#[harmos::operation(resources = [crate::hosting::Host<PauseExecution>], response = ControlReply, id = "dokime.execution.pause_execution",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "PauseExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseExecution {
    pub execution_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(PauseExecution);
}

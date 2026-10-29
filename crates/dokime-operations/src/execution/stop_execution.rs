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

#[harmos::operation(resources = [crate::hosting::Host<StopExecution>], response = ControlReply, id = "dokime.execution.stop_execution",
        policy = MUTATION, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "StopExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StopExecution {
    pub execution_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(StopExecution);
}

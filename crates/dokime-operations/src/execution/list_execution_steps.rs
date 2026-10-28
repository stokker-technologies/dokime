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

#[harmos::operation(resources = [crate::hosting::Host<ListExecutionSteps>], response = ListExecutionStepsReply, id = "dokime.execution.list_execution_steps",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListExecutionSteps"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionSteps {
    pub execution_id: String,
    pub limit: u32,
    pub cursor: Option<StepPageCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListExecutionSteps);
}

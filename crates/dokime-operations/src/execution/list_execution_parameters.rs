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

#[harmos::operation(resources = [crate::hosting::Host<ListExecutionParameters>], response = ListMaterializedParametersReply, id = "dokime.execution.list_execution_parameters",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListExecutionParameters"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionParameters {
    pub execution_id: String,
    pub set: ExecutionParameterSet,
    pub limit: u32,
    pub after_name: String,
    pub cursor: Option<ParameterCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListExecutionParameters);
}

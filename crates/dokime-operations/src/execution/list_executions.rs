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

#[harmos::operation(resources = [crate::hosting::Host<ListExecutions>], response = ListExecutionsReply, id = "dokime.execution.list_executions",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListExecutions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutions {
    pub cycle_id: String,
    pub limit: u32,
    pub after_execution_id: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListExecutions);
}

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

#[harmos::operation(resources = [crate::hosting::Host<BrowseExecutions>], response = ExecutionBrowsePage, id = "dokime.history.browse_executions",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "BrowseExecutions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowseExecutions {
    pub limit: u32,
    pub after_execution_id: String,
    pub filter: Option<ExecutionFilter>,
    pub order: Option<ExecutionOrder>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(BrowseExecutions);
}

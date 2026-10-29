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

// `filter` and `order` must be repeated unchanged on every continuation, and
// `after_execution_id` must still be a member of the filtered result. A
// changed selection, a changed order, or a continuation ID the filter excludes
// is rejected rather than silently restarted.
#[harmos::operation(resources = [crate::hosting::Host<BrowseExecutions>], response = ExecutionBrowsePage, id = "dokime.history.browse_executions",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
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

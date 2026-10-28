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

#[harmos::operation(resources = [crate::hosting::Host<DeleteCase>], response = ControlReply, id = "dokime.execution.delete_case",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "DeleteCase"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCase {
    pub cycle_id: String,
    pub case_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DeleteCase);
}

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

#[harmos::operation(resources = [crate::hosting::Host<UpdateCyclePolicy>], response = ControlReply, id = "dokime.execution.update_cycle_policy",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "UpdateCyclePolicy"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCyclePolicy {
    pub cycle_id: String,
    pub continue_automatically: bool,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdateCyclePolicy);
}

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

#[harmos::operation(resources = [crate::hosting::Host<PauseCycle>], response = ControlReply, id = "dokime.execution.pause_cycle",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "PauseCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseCycle {
    pub cycle_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(PauseCycle);
}

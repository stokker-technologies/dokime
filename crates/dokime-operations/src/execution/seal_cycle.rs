/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Seal A Cycle After Its Executions Become Durable

use serde::{Deserialize, Serialize};

use super::*;
use crate::TRANSFER;

#[harmos::operation(resources = [crate::hosting::Host<SealCycle>], response = ControlReply, id = "dokime.execution.seal_cycle",
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Execution", method = "SealCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SealCycle {
    pub cycle_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(SealCycle);
}

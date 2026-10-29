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

#[harmos::operation(resources = [crate::hosting::Host<ResumeCycle>], response = ControlReply, id = "dokime.execution.resume_cycle",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ResumeCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumeCycle {
    pub cycle_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ResumeCycle);
}

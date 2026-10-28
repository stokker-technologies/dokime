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

#[harmos::operation(resources = [crate::hosting::Host<CopyCycle>], response = CreateCycleReply, id = "dokime.execution.copy_cycle",
        policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "CopyCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyCycle {
    pub source_cycle_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CopyCycle);
}

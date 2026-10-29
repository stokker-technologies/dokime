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

#[harmos::operation(resources = [crate::hosting::Host<CreateCycle>], response = CreateCycleReply, id = "dokime.execution.create_cycle",
        policy = MUTATION, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "CreateCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCycle {
    #[row(
        description = "The label is required and nonblank; an empty identifier is derived by the backend. Description, tags and attributes are optional."
    )]
    pub metadata: Option<Metadata>,

    pub continue_automatically: bool,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CreateCycle);
}

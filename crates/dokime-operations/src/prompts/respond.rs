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

#[harmos::operation(resources = [crate::hosting::Host<Respond>], response = PromptResponseReceipt, id = "dokime.prompts.respond",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "Respond"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Respond {
    pub prompt_id: String,
    pub action: String,
    pub values: Option<Value>,
    pub accepted: bool,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(Respond);
}

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

#[harmos::operation(resources = [crate::hosting::Host<GetPrompt>], response = PromptView, id = "dokime.prompts.get_prompt",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "GetPrompt"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPrompt {
    pub prompt_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetPrompt);
}

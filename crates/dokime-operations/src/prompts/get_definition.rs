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

#[harmos::operation(resources = [crate::hosting::Host<GetDefinition>], response = PromptDefinition, id = "dokime.prompts.get_definition",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "GetDefinition"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetDefinition {
    pub prompt_type: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetDefinition);
}

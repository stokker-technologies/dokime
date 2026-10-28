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

#[harmos::operation(resources = [crate::hosting::Host<ListDefinitions>], response = ListPromptDefinitionsReply, id = "dokime.prompts.list_definitions",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "ListDefinitions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListDefinitions {
    pub limit: u32,
    pub after_prompt_type: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListDefinitions);
}

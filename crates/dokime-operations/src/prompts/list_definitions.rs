/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Current definitions are ordered by prompt type; pending requests by prompt ID.
// Subsequent pages require the same runtime epoch/revision and selection.
#[harmos::operation(resources = [crate::hosting::Host<ListDefinitions>], response = ListPromptDefinitionsReply, id = "dokime.prompts.list_definitions",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "ListDefinitions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListDefinitions {
    pub limit: u32,
    pub after_prompt_type: String,
    pub cursor: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListDefinitions);
}

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

// Responder identity comes from the authenticated principal, never a request
// label. This mutation follows MutationContext's no-automatic-retry contract.
// After an ambiguous failure, GetPrompt resolves whether an answer was recorded.
#[harmos::operation(resources = [crate::hosting::Host<Respond>], response = PromptResponseReceipt, id = "dokime.prompts.respond",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "Respond"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Respond {
    pub prompt_id: String,
    pub action: String,
    pub values: Option<Value>,
    pub accepted: bool,
    pub context: Option<MutationContext>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

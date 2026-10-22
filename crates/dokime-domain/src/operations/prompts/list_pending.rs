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

#[harmos_domain::message]
#[harmos_domain::operation(response = ListPendingPromptsReply, id = "dokime.prompts.list_pending", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Prompts", method = "ListPending"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPending {
    pub limit: u32,
    pub after_prompt_id: String,
    pub cursor: Option<Cursor>,
    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

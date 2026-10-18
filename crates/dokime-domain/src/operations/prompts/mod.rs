/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod get_definition;
mod get_prompt;
mod list_definitions;
mod list_pending;
mod respond;

/* === Re-Exports === */
pub use get_definition::GetDefinition;
pub use get_prompt::GetPrompt;
pub use list_definitions::ListDefinitions;
pub use list_pending::ListPending;
pub use respond::Respond;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
#[allow(unused_imports)]
use crate::model::execution::wire::*;
use crate::model::execution::wire_cursor::Cursor;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::values::wire::Value;

use crate::model::prompts::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Current definitions are ordered by prompt type; pending requests by prompt ID.
// Subsequent pages require the same runtime epoch/revision and selection.

// Responder identity comes from the authenticated principal, never a request
// label. This mutation follows MutationContext's no-automatic-retry contract.
// After an ambiguous failure, GetPrompt resolves whether an answer was recorded.

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ListDefinitions(ListDefinitions),
    GetDefinition(GetDefinition),
    ListPending(ListPending),
    GetPrompt(GetPrompt),
    Respond(Respond),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListDefinitions {
    const PATH: &'static str = "/dokime.v1.Prompts/ListDefinitions";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetDefinition {
    const PATH: &'static str = "/dokime.v1.Prompts/GetDefinition";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListPending {
    const PATH: &'static str = "/dokime.v1.Prompts/ListPending";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetPrompt {
    const PATH: &'static str = "/dokime.v1.Prompts/GetPrompt";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for Respond {
    const PATH: &'static str = "/dokime.v1.Prompts/Respond";
    const ACCESS: Access = Access::Control;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

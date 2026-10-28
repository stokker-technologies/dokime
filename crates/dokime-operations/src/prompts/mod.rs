/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod get_definition;
mod get_prompt;
mod list_definitions;
mod list_pending;
mod respond;

pub use get_definition::GetDefinition;
pub use get_prompt::GetPrompt;
pub use list_definitions::ListDefinitions;
pub use list_pending::ListPending;
pub use respond::Respond;

use super::Endpoint;
use super::ORDINARY;

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::execution::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::prompts::wire::*;
use dokime_domain::model::values::wire::Value;

// Current definitions are ordered by prompt type; pending requests by prompt ID.

// Subsequent pages require the same runtime epoch/revision and selection.

// Responder identity comes from the authenticated principal, never a request

// label. This mutation follows MutationContext's no-automatic-retry contract.

// After an ambiguous failure, GetPrompt resolves whether an answer was recorded.

#[harmos::catalog]
pub enum Operations {
    ListDefinitions(ListDefinitions),
    GetDefinition(GetDefinition),
    ListPending(ListPending),
    GetPrompt(GetPrompt),
    Respond(Respond),
}

impl Endpoint for ListDefinitions {
    const PATH: &'static str = "/dokime.v1.Prompts/ListDefinitions";
}

impl Endpoint for GetDefinition {
    const PATH: &'static str = "/dokime.v1.Prompts/GetDefinition";
}

impl Endpoint for ListPending {
    const PATH: &'static str = "/dokime.v1.Prompts/ListPending";
}

impl Endpoint for GetPrompt {
    const PATH: &'static str = "/dokime.v1.Prompts/GetPrompt";
}

impl Endpoint for Respond {
    const PATH: &'static str = "/dokime.v1.Prompts/Respond";
}

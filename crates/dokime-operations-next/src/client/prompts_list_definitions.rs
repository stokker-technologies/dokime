/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Prompts List Definitions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

pub const ID: &str = "dokime.prompts.list_definitions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PromptsListDefinitionsRequest, v1::ListPromptDefinitionsReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsListDefinitionsRequest) -> Result<v1::ListPromptDefinitionsReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let definitions = super::prompt_catalog::definitions(context)?;
        let cursor = super::paging::cursor(context, "prompts.definitions".into(), definitions.iter().map(Message::encode_to_vec));
        let (definitions, next_after_prompt_type) = super::paging::page(definitions, request.limit, &request.after_prompt_type, request.cursor.as_ref(), &cursor, context.settings.page_items, |definition| &definition.metadata.as_ref().unwrap().identifier)?;
        let definitions = definitions.into_iter().map(|definition| { let metadata = definition.metadata.unwrap(); v1::PromptDefinitionIdentity { prompt_type: metadata.identifier, title: metadata.label } }).collect();
        Ok(v1::ListPromptDefinitionsReply { definitions, next_after_prompt_type, cursor: Some(cursor) })
    }).await
}

/* === Private Functions === */

/* === Tests === */

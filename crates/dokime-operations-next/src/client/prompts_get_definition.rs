/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Get Definition

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.prompts.get_definition";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PromptsGetDefinitionRequest, v1::Prompt>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsGetDefinitionRequest) -> Result<v1::Prompt> {
    super::complete(ID, context.settings.read_timeout, async {
        super::prompt_catalog::definitions(context)?
            .into_iter()
            .find(|definition| definition.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.prompt_type))
            .ok_or_else(|| Error::NotFound("Get Prompt Definition: Select a Type Returned by ListDefinitions".into()))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

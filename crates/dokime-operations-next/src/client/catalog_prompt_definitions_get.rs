/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Prompt Definitions Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.PromptDefinitions/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::PromptDefinitionsGetRequest, dokime_protocol::client::PromptDefinitionsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PromptDefinitionsGetRequest,
) -> Result<dokime_protocol::client::PromptDefinitionsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let reply = super::catalog_prompt_definitions_list::handle(
            context,
            dokime_protocol::client::PromptDefinitionsListRequest { parent: request.parent, at: request.at },
        )
        .await?;
        let value = reply
            .values
            .into_iter()
            .find(|value| value.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.name))
            .ok_or_else(|| Error::NotFound("Get Prompt Definition: Select a Declared Identifier".into()))?;
        Ok(dokime_protocol::client::PromptDefinitionsGetResponse {
            value: Some(value),
            resolved_at: reply.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

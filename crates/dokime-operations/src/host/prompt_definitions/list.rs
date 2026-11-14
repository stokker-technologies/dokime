/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Prompt Definitions List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.PromptDefinitions/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::PromptDefinitionsListRequest, dokime_protocol::client::PromptDefinitionsListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PromptDefinitionsListRequest,
) -> Result<dokime_protocol::client::PromptDefinitionsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = super::views::read(context, &format!("catalog/prompts/{}", request.parent), request.at.as_ref(), async {
            Ok(dokime_protocol::client::PromptDefinitionsListResponse {
                values: super::prompt_catalog::definitions(context)?,
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

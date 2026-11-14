/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Prompts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::PromptsGetRequest, dokime_protocol::client::PromptsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PromptsGetRequest,
) -> Result<dokime_protocol::client::PromptsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (prompt, cut) = super::views::read(context, &format!("prompt/{}", request.name), request.at.as_ref(), async {
            super::prompt_catalog::view(context, &request.name)
        })
        .await?;
        Ok(dokime_protocol::client::PromptsGetResponse {
            prompt: Some(prompt),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

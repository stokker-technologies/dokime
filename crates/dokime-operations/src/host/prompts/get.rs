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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::PromptsGetRequest, dokime_protocol::host::PromptsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::PromptsGetRequest,
) -> Result<dokime_protocol::host::PromptsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (prompt, cut) = crate::host::views::read(context, &format!("prompt/{}", request.name), request.at.as_ref(), async {
            crate::host::prompts::catalog::view(context, &request.name)
        })
        .await?;
        Ok(dokime_protocol::host::PromptsGetResponse {
            prompt: Some(prompt),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

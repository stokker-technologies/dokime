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

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::PromptsGetRequest;
use dokime_protocol::host::PromptsGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<PromptsGetRequest, PromptsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: PromptsGetRequest) -> Result<PromptsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (prompt, cut) = crate::host::views::read(context, &format!("prompt/{}", request.name), request.at.as_ref(), async {
            crate::host::prompts::catalog::view(context, &request.name)
        })
        .await?;
        Ok(PromptsGetResponse {
            prompt: Some(prompt),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(PromptsGetRequest, PromptsGetResponse, context, "dokime.host.v1", "Prompts", "Get");

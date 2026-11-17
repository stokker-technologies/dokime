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
use crate::{Error, Result, host::Context};
use dokime_protocol::host::PromptsGetRequest;
use dokime_protocol::host::PromptsGetResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: PromptsGetRequest) -> Result<PromptsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (execution, _) =
            request.name.split_once('/').ok_or_else(|| Error::Invalid("Get Prompt: Select Its Returned Identifier".into()))?;
        let owner = Scope::Execution(execution).identifier()?;
        let (prompt, cut) = crate::host::views::read(context, owner, &format!("prompt/{}", request.name), request.at.as_ref(), async {
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

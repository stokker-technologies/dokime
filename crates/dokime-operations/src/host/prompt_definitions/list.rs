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
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::PromptDefinitionsListRequest;
use dokime_protocol::host::PromptDefinitionsListResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.PromptDefinitions/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<PromptDefinitionsListRequest, PromptDefinitionsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: PromptDefinitionsListRequest) -> Result<PromptDefinitionsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let owner = Scope::Root(&context.host.identity().backend_id).identifier()?;
        let (mut reply, cut) =
            crate::host::views::read(context, owner, &format!("catalog/prompts/{}", request.parent), request.at.as_ref(), async {
                Ok(PromptDefinitionsListResponse {
                    values: crate::host::prompts::catalog::definitions(context).await?,
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

crate::native::unary!(PromptDefinitionsListRequest, PromptDefinitionsListResponse, context, "dokime.host.v1", "PromptDefinitions", "List");

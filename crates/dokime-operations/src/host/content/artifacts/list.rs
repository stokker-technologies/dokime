/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsListResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ArtifactsListRequest) -> Result<ArtifactsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!("artifacts/{}", crate::host::views::hex(&request.scope.as_ref().map(Message::encode_to_vec).unwrap_or_default()));
        let owner = if let Some(execution) = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) {
            Scope::Execution(execution).identifier()?
        } else if let Some(cycle) = request.scope.as_ref().and_then(|scope| scope.cycle_id.as_deref()) {
            Scope::Cycle(cycle).identifier()?
        } else {
            Scope::Root(&context.host.identity().backend_id).identifier()?
        };
        let (saved, cut) = crate::host::views::read(context, owner, &key, request.at.as_ref(), async {
            let artifacts = crate::host::content::artifact_records(context)
                .await?
                .into_iter()
                .filter_map(|stored| stored.artifact)
                .filter(|artifact| {
                    request.scope.as_ref().is_none_or(|selected| {
                        artifact.scope.as_ref().is_some_and(|scope| {
                            selected.cycle_id.as_ref().is_none_or(|id| scope.cycle_id.as_ref() == Some(id))
                                && selected.execution_id.as_ref().is_none_or(|id| scope.execution_id.as_ref() == Some(id))
                                && selected.case_id.as_ref().is_none_or(|id| scope.case_id.as_ref() == Some(id))
                        })
                    })
                })
                .collect();
            Ok(ArtifactsListResponse { artifacts, ..Default::default() })
        })
        .await?;
        let (artifacts, next_page_token) = crate::host::views::page(
            saved.artifacts,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        crate::host::bounded_reply(
            context,
            ArtifactsListResponse {
                artifacts,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ArtifactsListRequest, ArtifactsListResponse, context, "dokime.host.v1.content", "Artifacts", "List");

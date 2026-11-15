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

use crate::host::ClientSettings;
use prost::Message;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsListResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ArtifactsListRequest, ArtifactsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ArtifactsListRequest) -> Result<ArtifactsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!("artifacts/{}", crate::host::views::hex(&request.scope.as_ref().map(Message::encode_to_vec).unwrap_or_default()));
        let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let store = context.host.store();
            let mut after = String::new();
            let mut artifacts = Vec::new();
            loop {
                let page = store.artifacts_page(&after, context.settings.page_items)?;
                if page.is_empty() {
                    break;
                }
                after = page.last().unwrap().artifact_id.clone();
                artifacts.extend(page.into_iter().filter(|artifact| {
                    request.scope.as_ref().is_none_or(|selected| {
                        artifact.scope.as_ref().is_some_and(|scope| {
                            selected.cycle_id.as_ref().is_none_or(|id| scope.cycle_id.as_ref() == Some(id))
                                && selected.execution_id.as_ref().is_none_or(|id| scope.execution_id.as_ref() == Some(id))
                                && selected.case_id.as_ref().is_none_or(|id| scope.case_id.as_ref() == Some(id))
                        })
                    })
                }));
            }
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

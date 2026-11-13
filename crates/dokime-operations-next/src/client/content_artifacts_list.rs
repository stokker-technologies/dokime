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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.content.Artifacts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::content::ArtifactsListRequest, dokime_protocol::client::content::ArtifactsListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::content::ArtifactsListRequest,
) -> Result<dokime_protocol::client::content::ArtifactsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        super::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key =
            format!("artifacts/{}", super::views::hex(&request.scope.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default()));
        let (saved, cut) = super::views::read(context, &key, request.at.as_ref(), async {
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
            Ok(dokime_protocol::client::content::ArtifactsListResponse { artifacts, ..Default::default() })
        })
        .await?;
        let (artifacts, next_page_token) =
            super::views::page(saved.artifacts, &cut, request.page.as_ref(), context.settings.page_items, context.settings.message_bytes)?;
        super::bounded_reply(
            context,
            dokime_protocol::client::content::ArtifactsListResponse {
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

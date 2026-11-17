/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::Failure;

// Local Imports
use crate::host::Output;
use crate::{Result, host::Context};
use dokime_protocol::host::content::ArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsWatchRequest;
use dokime_protocol::host::content::ArtifactsWatchResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ArtifactsWatchRequest) -> Result<Output<ArtifactsWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut runtime = context.host.changes();
        let mut selection = ArtifactsListRequest {
            scope: request.scope,
            at: request.at,
            page: request.page,
        };
        let first = crate::host::content::artifacts::list::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = ArtifactsWatchResponse {
                        artifacts: page.artifacts,
                        resolved_at: cut.clone(),
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return;
                    }
                    if !token.is_empty() {
                        selection.at = cut;
                        selection.page.get_or_insert_default().page_token = token;
                    } else {
                        if context.host.shutting_down() {
                            return;
                        }
                        tokio::select! {
                            _ = sender.closed() => return,
                            changed = runtime.changed() => if changed.is_err() { return; },
                        }
                        opening = false;
                        selection.at = None;
                        selection.page.get_or_insert_default().page_token.clear();
                    }
                    page = match crate::host::content::artifacts::list::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(ArtifactsWatchResponse {
                                    resync_required: Some(Failure {
                                        code: "resync_required".into(),
                                        message: format!("{ID}: {error}"),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }))
                                .await;
                            return;
                        }
                    };
                }
            };
            if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded(format!("{ID}: Reopen After Its Deadline"))));
            }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::download!(ArtifactsWatchRequest, ArtifactsWatchResponse, "dokime.host.v1.content", "Artifacts", "Watch");

/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Prompts/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::PromptsWatchRequest, dokime_protocol::client::PromptsWatchResponse>(
        ID,
        settings.watch_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::PromptsWatchRequest,
) -> Result<super::Output<dokime_protocol::client::PromptsWatchResponse>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = dokime_protocol::client::PromptsListRequest {
            parent: request.parent,
            cycle_id: request.cycle_id,
            execution_id: request.execution_id,
            pending_only: request.pending_only,
            at: request.at,
            page: request.page,
        };
        let first = super::prompts_list::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = dokime_protocol::client::PromptsWatchResponse {
                        prompts: page.prompts,
                        resolved_at: cut.clone(),
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return;
                    }
                    if !token.is_empty() {
                        selection.at = cut;
                        if !selection.pending_only {
                            selection.page.get_or_insert_default().page_token = token;
                        }
                    } else {
                        if context.host.shutting_down() {
                            return;
                        }
                        tokio::select! {
                            _ = sender.closed() => return,
                            changed = storage.changed() => if changed.is_err() { return; },
                            changed = runtime.changed() => if changed.is_err() { return; },
                        }
                        opening = false;
                        selection.at = None;
                        if let Some(page) = &mut selection.page {
                            page.page_token.clear();
                        }
                    }
                    page = match super::prompts_list::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(dokime_protocol::client::PromptsWatchResponse {
                                    resync_required: Some(h::Failure {
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

/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Documents Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Documents/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::DocumentsWatchRequest, dokime_protocol::client::DocumentsWatchResponse>(
        ID,
        settings.watch_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::DocumentsWatchRequest,
) -> Result<super::Output<dokime_protocol::client::DocumentsWatchResponse>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = dokime_protocol::client::DocumentsGetRequest {
            scope: request.scope,
            document_id: request.document_id,
            structure_only: request.structure_only,
            card_ids: request.card_ids,
            page_token: request.page_token,
            max_bytes: request.max_bytes,
            at: request.at,
        };
        let first = super::documents_get::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = dokime_protocol::client::DocumentsWatchResponse {
                        frames: page.frames,
                        resolved_at: cut.clone(),
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return;
                    }
                    if !token.is_empty() {
                        selection.at = cut;
                        selection.page_token = token;
                    } else {
                        if context.host.shutting_down()
                            || selection.scope.as_ref().is_some_and(|scope| {
                                !scope.execution_id.is_empty()
                                    && context
                                        .host
                                        .store()
                                        .metadata(&scope.execution_id)
                                        .is_ok_and(|cut| cut.state != dokime_storage::ExecutionState::Open)
                            })
                        {
                            return;
                        }
                        tokio::select! {
                            _ = sender.closed() => return,
                            changed = storage.changed() => if changed.is_err() { return; },
                            changed = runtime.changed() => if changed.is_err() { return; },
                        }
                        opening = false;
                        selection.at = None;
                        selection.page_token.clear();
                    }
                    page = match super::documents_get::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(dokime_protocol::client::DocumentsWatchResponse {
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

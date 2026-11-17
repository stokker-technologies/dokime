/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Execution Files Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::Failure;

// Local Imports
use crate::host::Output;
use crate::{Result, host::Context};
use dokime_protocol::host::content::ExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesWatchRequest;
use dokime_protocol::host::content::ExecutionFilesWatchResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.ExecutionFiles/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionFilesWatchRequest) -> Result<Output<ExecutionFilesWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = ExecutionFilesListRequest {
            parent: request.parent,
            at: request.at,
            page: request.page,
        };
        let first = crate::host::content::execution_files::list::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = ExecutionFilesWatchResponse {
                        files: page.files,
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
                            changed = storage.changed() => if changed.is_err() { return; },
                            changed = runtime.changed() => if changed.is_err() { return; },
                        }
                        opening = false;
                        selection.at = None;
                        selection.page.get_or_insert_default().page_token.clear();
                    }
                    page = match crate::host::content::execution_files::list::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(ExecutionFilesWatchResponse {
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

crate::native::download!(ExecutionFilesWatchRequest, ExecutionFilesWatchResponse, "dokime.host.v1.content", "ExecutionFiles", "Watch");

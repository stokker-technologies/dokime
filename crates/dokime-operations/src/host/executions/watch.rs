/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::host::Output;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsWatchRequest;
use dokime_protocol::host::ExecutionsWatchResponse;
use harmos_protocol::model::Failure;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsWatchRequest, ExecutionsWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsWatchRequest) -> Result<Output<ExecutionsWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = ExecutionsListRequest {
            parent: request.parent,
            selection: request.selection,
            filter: request.filter,
            order: request.order,
            at: request.at,
            page: request.page,
        };
        let first = crate::host::executions::list::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = ExecutionsWatchResponse {
                        executions: page.executions,
                        outcome_counts: page.outcome_counts,
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
                    page = match crate::host::executions::list::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(ExecutionsWatchResponse {
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

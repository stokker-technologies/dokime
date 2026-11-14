/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::host::Output;
use crate::{Result, host::Context};
use dokime_protocol::host::CasesListRequest;
use dokime_protocol::host::CasesWatchRequest;
use dokime_protocol::host::CasesWatchResponse;
use harmos_protocol::model::Failure;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesWatchRequest, CasesWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: CasesWatchRequest) -> Result<Output<CasesWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = CasesListRequest { parent: request.parent, at: request.at };
        let first = crate::host::cases::list::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = String::new();
                    let response = CasesWatchResponse {
                        cases: page.cases,
                        resolved_at: cut.clone(),
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return;
                    }
                    if !token.is_empty() {
                        selection.at = cut;
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
                    }
                    page = match crate::host::cases::list::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(CasesWatchResponse {
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

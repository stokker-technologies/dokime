/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Overview Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::Output;

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::backend::OverviewGetRequest;
use dokime_protocol::host::backend::OverviewWatchRequest;
use dokime_protocol::host::backend::OverviewWatchResponse;

use harmos_protocol::model::OperationContract;

use harmos_protocol::model::Failure;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.backend.Overview/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<OverviewWatchRequest, OverviewWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: OverviewWatchRequest) -> Result<Output<OverviewWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = OverviewGetRequest { at: request.at };
        let first = crate::host::backend::overview::get::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = String::new();
                    let response = OverviewWatchResponse {
                        overview: page.overview,
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
                    page = match crate::host::backend::overview::get::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            let _ = sender
                                .send(Ok(OverviewWatchResponse {
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

crate::native::download!(OverviewWatchRequest, OverviewWatchResponse, "dokime.host.v1.backend", "Overview", "Watch");

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

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::Failure;
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use crate::Result;
use crate::host::{Context, Output};
use dokime_protocol::host::backend::{OverviewGetRequest, OverviewWatchRequest, OverviewWatchResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.backend.Overview/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: OverviewWatchRequest) -> Result<Output<OverviewWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = OverviewGetRequest { at: request.at };
        let first = crate::host::backend::overview::get::handle(context, selection.clone()).await?;
        let context = context.clone();
        let (sender, output) = mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at;
                    // The Watch Ends After Sending a Snapshot Read While the Host Shuts Down, Never Before.
                    let stopping = page.overview.as_ref().is_some_and(|overview| overview.shutting_down);
                    let response = OverviewWatchResponse {
                        overview: page.overview,
                        resolved_at: cut,
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return;
                    }
                    if stopping {
                        return;
                    }
                    tokio::select! {
                        _ = sender.closed() => return,
                        changed = storage.changed() => if changed.is_err() { return; },
                        changed = runtime.changed() => if changed.is_err() { return; },
                    }
                    opening = false;
                    selection.at = None;
                    page = match crate::host::backend::overview::get::handle(&context, selection.clone()).await {
                        Ok(page) => page,
                        Err(error) => {
                            tracing::warn!(error = %error, "Publish Runtime Overview: Resynchronize the Watch After Its Snapshot Failed");
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
            if time::timeout(context.settings.watch_timeout, work).await.is_err() {
                let _ = sender.try_send(Err(Status::deadline_exceeded(format!("{ID}: Reopen After Its Deadline"))));
            }
        });
        Ok(ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::download!(OverviewWatchRequest, OverviewWatchResponse, "dokime.host.v1.backend", "Overview", "Watch");

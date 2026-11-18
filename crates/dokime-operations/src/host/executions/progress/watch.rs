/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use crate::Error;
use crate::{
    Result,
    host::{Context, Output},
};
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressWatchRequest;
use dokime_protocol::host::executions::ProgressWatchResponse;
use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Progress/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ProgressWatchRequest) -> Result<Output<ProgressWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut runtime = context.host.progress(&request.name)?;
        let mut storage = context.host.store().subscribe();
        let (sender, output) = mpsc::channel(context.settings.stream_buffer.saturating_add(1));
        let terminal =
            sender.clone().reserve_owned().await.map_err(|_| Error::Invalid("Watch Progress: Reconnect the Client Stream".into()))?;
        let context = context.clone();
        tokio::spawn(async move {
            let timeout = context.settings.watch_timeout;
            let work = async {
                let mut at = request.at;
                let mut previous: Option<ProgressWatchResponse> = None;
                loop {
                    let live = runtime.borrow_and_update().clone();
                    let pinned_opening = at.is_some();
                    let cut = match context.host.store().cut(&request.name) {
                        Ok(cut) => cut,
                        Err(error) => {
                            let _ = sender.send(Err(crate::host::status(error.into()))).await;
                            return;
                        }
                    };
                    let sealed = !pinned_opening && cut.state == ExecutionState::Sealed;
                    if !sealed && let Some(failure) = live.failure {
                        let _ = sender.send(Err(Status::internal(format!("Watch Progress: {failure}")))).await;
                        return;
                    }
                    let saved = match crate::host::executions::progress::get::handle(
                        &context,
                        ProgressGetRequest {
                            name: Some(request.name.clone()),
                            at: at.take(),
                        },
                    )
                    .await
                    {
                        Ok(saved) => saved,
                        Err(error) => {
                            let _ = sender.send(Err(crate::host::status(error))).await;
                            return;
                        }
                    };
                    let outcome = if sealed { cut.execution.outcome } else { None };
                    if sealed && outcome.as_ref().is_none_or(|outcome| outcome.attribution.is_none()) {
                        let _ = sender
                            .send(Err(Status::failed_precondition("Watch Progress: Restore the Sealed Execution's Stored Outcome")))
                            .await;
                        return;
                    }
                    let response = ProgressWatchResponse {
                        value: saved.value,
                        revision: saved.revision,
                        resolved_at: saved.resolved_at,
                        phase: if sealed { "sealed".into() } else { live.phase },
                        finished: sealed,
                        sealed,
                        outcome,
                    };
                    let unchanged = previous.as_ref().is_some_and(|previous| {
                        previous.value == response.value
                            && previous.revision == response.revision
                            && previous.phase == response.phase
                            && previous.finished == response.finished
                            && previous.sealed == response.sealed
                            && previous.outcome == response.outcome
                    });
                    if !unchanged {
                        previous = Some(response.clone());
                        if sender.send(Ok(response)).await.is_err() {
                            return;
                        }
                    }
                    if pinned_opening {
                        continue;
                    }
                    if sealed {
                        return;
                    }
                    if live.finished {
                        let _ = sender
                            .send(Err(Status::failed_precondition("Watch Progress: Inspect the Execution; Its Owner Stopped Before Seal")))
                            .await;
                        return;
                    }
                    tokio::select! {
                        _ = sender.closed() => return,
                        changed = runtime.changed() => if changed.is_err() {
                            let _ = sender.send(Err(Status::unavailable("Watch Progress: Execution Owner Closed Before Settlement"))).await;
                            return;
                        },
                        changed = storage.changed() => if changed.is_err() {
                            let _ = sender.send(Err(Status::unavailable("Watch Progress: Storage Closed Before Settlement"))).await;
                            return;
                        },
                    }
                }
            };
            if time::timeout(timeout, work).await.is_err() {
                terminal.send(Err(Status::deadline_exceeded("Watch Progress: Operation Deadline Expired")));
            }
        });
        Ok(ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::download!(ProgressWatchRequest, ProgressWatchResponse, "dokime.host.v1.executions", "Progress", "Watch");

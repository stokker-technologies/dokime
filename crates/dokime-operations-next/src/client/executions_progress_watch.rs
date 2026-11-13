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

// Local Imports
use crate::{
    Result,
    client::{Context, Output},
};
use dokime_protocol::v1::executions::{ProgressGetRequest, ProgressWatchRequest, ProgressWatchResponse};

/* === Constants === */

pub const ID: &str = "dokime.v1.executions.Progress/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<ProgressWatchRequest, ProgressWatchResponse>(ID, settings.watch_timeout)
}

pub async fn handle(context: &Context, request: ProgressWatchRequest) -> Result<Output<ProgressWatchResponse>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut runtime = context.host.progress(&request.name)?;
        let mut storage = context.host.store().subscribe();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer.saturating_add(1));
        let terminal = sender.clone().reserve_owned().await.map_err(|_| crate::Error::Invalid("Watch Progress: Reconnect the Client Stream".into()))?;
        let context = context.clone();
        tokio::spawn(async move {
            let timeout = context.settings.watch_timeout;
            let work = async {
                let mut at = request.at;
                let mut previous: Option<ProgressWatchResponse> = None;
                loop {
                    let live = runtime.borrow_and_update().clone();
                    if let Some(failure) = live.failure {
                        let _ = sender.send(Err(tonic::Status::internal(format!("Watch Progress: {failure}")))).await;
                        return;
                    }
                    let pinned_opening = at.is_some();
                    let saved = match super::executions_progress_get::handle(&context, ProgressGetRequest { name: request.name.clone(), at: at.take() }).await {
                        Ok(saved) => saved,
                        Err(error) => { let _ = sender.send(Err(super::status(error))).await; return; }
                    };
                    let response = ProgressWatchResponse { value: saved.value, revision: saved.revision, resolved_at: saved.resolved_at, phase: live.phase, finished: live.finished };
                    let unchanged = previous.as_ref().is_some_and(|previous| {
                        previous.value == response.value && previous.revision == response.revision
                            && previous.phase == response.phase && previous.finished == response.finished
                    });
                    if !unchanged {
                        previous = Some(response.clone());
                        if sender.send(Ok(response)).await.is_err() { return; }
                    }
                    if pinned_opening { continue; }
                    if live.finished { return; }
                    tokio::select! {
                        _ = sender.closed() => return,
                        changed = runtime.changed() => if changed.is_err() {
                            let _ = sender.send(Err(tonic::Status::unavailable("Watch Progress: Execution Owner Closed Before Settlement"))).await;
                            return;
                        },
                        changed = storage.changed() => if changed.is_err() {
                            let _ = sender.send(Err(tonic::Status::unavailable("Watch Progress: Storage Closed Before Settlement"))).await;
                            return;
                        },
                    }
                }
            };
            if tokio::time::timeout(timeout, work).await.is_err() {
                terminal.send(Err(tonic::Status::deadline_exceeded("Watch Progress: Operation Deadline Expired")));
            }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(output))
    }).await
}

/* === Private Functions === */

/* === Tests === */

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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.observation.watch_execution_progress";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ObservationWatchExecutionProgressRequest, v1::ObservationWatchExecutionProgressOutput>(
        ID,
        settings.watch_timeout,
    )
}
pub async fn handle(
    context: &Context, request: v1::ObservationWatchExecutionProgressRequest,
) -> Result<Output<v1::ObservationWatchExecutionProgressOutput>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut receiver = context.host.progress(&request.test_execution_id)?;
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        let identifier = request.test_execution_id;
        let context = context.clone();
        let timeout = context.settings.watch_timeout;
        tokio::spawn(async move {
            let work = async {
                let mut initial = true;
                loop {
                    let progress = receiver.borrow_and_update().clone();
                    if let Some(failure) = progress.failure { let _ = sender.send(Err(tonic::Status::internal(format!("WatchExecutionProgress: {failure}")))).await; return; }
                    let snapshot = match super::execution_state::evidence(&context,&identifier) { Ok(snapshot) => snapshot, Err(error) => { let _ = sender.send(Err(tonic::Status::internal(format!("WatchExecutionProgress: {error}")))).await; return; } };
                    let saved = snapshot.progress().ok().and_then(|progress| progress.last().cloned()).unwrap_or_default();
                    let frame = v1::ExecutionProgressFrame { revision: progress.revision, resynchronized: initial, cleared: progress.finished,
                        progress: Some(v1::ExecutionProgress { test_execution_id: identifier.clone(), phase: progress.phase,
                            completed: saved.completed, total: saved.total, ..Default::default() }), ..Default::default() };
                    let frame = v1::ObservationWatchExecutionProgressOutput { value: Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) };
                    if sender.send(Ok(frame)).await.is_err() { return; }
                    initial = false;
                    if progress.finished {
                        let frame = v1::ObservationWatchExecutionProgressOutput { value: Some(v1::observation_watch_execution_progress_output::Value::Response(v1::ClientEmpty {})) };
                        let _ = sender.send(Ok(frame)).await;
                        return;
                    }
                    tokio::select! {
                        _ = sender.closed() => return,
                        changed = receiver.changed() => if changed.is_err() { let _ = sender.send(Err(tonic::Status::unavailable("WatchExecutionProgress: Host Worker Closed Before Completion"))).await; return; }
                    }
                }
            };
            if tokio::time::timeout(timeout, work).await.is_err() { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("WatchExecutionProgress: Operation Deadline Expired"))); }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(output))
    }).await
}

/* === Private Functions === */

/* === Tests === */

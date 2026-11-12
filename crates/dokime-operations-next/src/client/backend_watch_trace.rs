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
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.backend.watch_trace";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::WatchBackendTrace, v1::BackendWatchTraceOutput>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::WatchBackendTrace) -> Result<super::Output<v1::BackendWatchTraceOutput>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut changes = context.host.diagnostic_changes();
        let first = super::backend_streams::trace(context, &request)?;
        let context = context.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut request = request;
                let mut batch = first;
                loop {
                    request.after_sequence = batch.next_sequence;
                    request.epoch = Some(batch.epoch.clone());
                    if sender.send(Ok(v1::BackendWatchTraceOutput { value: Some(v1::backend_watch_trace_output::Value::Item(batch)) })).await.is_err() { return; }
                    if !request.follow {
                        let _ = sender.send(Ok(v1::BackendWatchTraceOutput { value: Some(v1::backend_watch_trace_output::Value::Response(v1::BackendWatchTraceCompletion {})) })).await;
                        return;
                    }
                    tokio::select! {
                        _ = sender.closed() => return,
                        result = changes.recv() => { if matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)) { return; } }
                    }
                    batch = match super::backend_streams::trace(&context, &request) { Ok(value) => value, Err(error) => { let _ = sender.send(Err(super::status(error))).await; return; } };
                }
            };
            if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Trace: Reconnect to Continue After the Watch Deadline"))); }
        });
        Ok(super::Output::new(receiver))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

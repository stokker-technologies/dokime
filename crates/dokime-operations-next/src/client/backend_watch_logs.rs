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

pub const ID: &str = "dokime.backend.watch_logs";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::WatchBackendLogs, v1::BackendWatchLogsOutput>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::WatchBackendLogs) -> Result<super::Output<v1::BackendWatchLogsOutput>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let minimum = super::backend_streams::level(request.min_level)?;
        let mut changes = context.host.diagnostic_changes();
        let (events, mut after, dropped) = context.host.diagnostic_events(None)?;
        let count = super::projection::limit(request.replay_lines, context.settings.page_items);
        let mut replay: Vec<_> = events.into_iter().filter(|event| event.level >= minimum).rev().take(count).collect();
        replay.reverse();
        let context = context.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                for (index, event) in replay.into_iter().enumerate() {
                    let item = v1::LogFrame { time: event.time, level: event.level, target: event.target, message: event.message, fields: event.fields, dropped: if index == 0 { dropped } else { 0 } };
                    if sender.send(Ok(v1::BackendWatchLogsOutput { value: Some(v1::backend_watch_logs_output::Value::Item(item)) })).await.is_err() { return; }
                }
                loop {
                    tokio::select! {
                        _ = sender.closed() => return,
                        result = changes.recv() => { if matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)) { return; } }
                    }
                    let (events, latest, dropped) = match context.host.diagnostic_events(None) { Ok(value) => value, Err(error) => { let _ = sender.send(Err(super::status(error))).await; return; } };
                    for event in events.into_iter().filter(|event| event.sequence > after && event.level >= minimum) {
                        let item = v1::LogFrame { time: event.time, level: event.level, target: event.target, message: event.message, fields: event.fields, dropped };
                        if sender.send(Ok(v1::BackendWatchLogsOutput { value: Some(v1::backend_watch_logs_output::Value::Item(item)) })).await.is_err() { return; }
                    }
                    after = latest;
                }
            };
            if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Logs: Reconnect to Continue After the Watch Deadline"))); }
        });
        Ok(super::Output::new(receiver))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

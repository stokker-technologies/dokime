/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Watch Stream Samples

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::{Context, Output}};
use dokime_protocol::v1;
use prost::Message;


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.telemetry.watch_samples";
pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::TelemetryWatchSamplesRequest, v1::TelemetryWatchSamplesOutput>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::TelemetryWatchSamplesRequest) -> Result<Output<v1::TelemetryWatchSamplesOutput>> {
    let selection = request.selection.ok_or_else(|| Error::Invalid("Watch Samples: Supply a Forward Selection".into()))?;
    if selection.latest { return Err(Error::Invalid("Watch Samples: Use Forward Paging Instead of a Latest Tail".into())); }
    let mut selection = v1::TelemetryListSamplesRequest::decode(selection.encode_to_vec().as_slice())?;
    let identifier = super::projection::execution_scope(selection.scope.as_ref())?.to_owned();
    let mut changes = context.host.store().subscribe();
    let first = super::telemetry_list_samples::handle(context, selection.clone()).await?;
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let mut page = first;
            loop {
                selection.after_stream_cursors = page.through_stream_cursors.iter().map(|cursor| v1::ListTelemetrySamplesRequestAfterStreamCursorsEntry { key: cursor.key.clone(), value: cursor.value }).collect();
                let complete = page.complete;
                if sender.send(Ok(v1::TelemetryWatchSamplesOutput { value: Some(v1::telemetry_watch_samples_output::Value::Item(page)) })).await.is_err() { return Ok::<(), Error>(()); }
                let sealed = context.host.store().metadata(&identifier)?.state != dokime_storage::ExecutionState::Open;
                if complete && (!request.follow || sealed) {
                    let _ = sender.send(Ok(v1::TelemetryWatchSamplesOutput { value: Some(v1::telemetry_watch_samples_output::Value::Response(v1::ClientEmpty {})) })).await;
                    return Ok(());
                }
                if complete {
                    tokio::select! {
                        _ = sender.closed() => return Ok(()),
                        changed = changes.changed() => if changed.is_err() { return Err(Error::Invalid("Watch Samples: Reconnect to the Host".into())); }
                    }
                }
                page = super::telemetry_list_samples::handle(&context, selection.clone()).await?;
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Err(error)) => { let _ = sender.send(Err(super::status(error))).await; }
            Err(_) => { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Samples: Reopen the Watch After Its Deadline"))); }
            _ => (),
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}


/* === Private Functions === */

/* === Tests === */

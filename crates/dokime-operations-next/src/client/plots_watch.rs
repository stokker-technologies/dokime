/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Watch Plot Windows

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{
    Error, Result,
    client::{Context, Output},
};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.plots.watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlotsWatchRequest, v1::PlotsWatchOutput>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::PlotsWatchRequest) -> Result<Output<v1::PlotsWatchOutput>> {
    let mut changes = context.host.store().subscribe();
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            loop {
                let mut settled = true;
                for request in &request.requests {
                    let request = request.clone();
                    let context = context.clone();
                    let key = request.plot_id.clone().unwrap_or_else(|| request.stream.clone());
                    if let Some(execution) = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) {
                        settled &= context.host.store().metadata(execution)?.state != dokime_storage::ExecutionState::Open;
                    }
                    let result = tokio::task::spawn_blocking(move || super::plots_query_snapshots::snapshot(&context, &request))
                        .await
                        .map_err(|error| Error::Invalid(error.to_string()))?;
                    let (kind, envelope) = match result {
                        Ok(envelope) => (v1::PlotStreamKind::Snapshot, envelope),
                        Err(error) => (
                            v1::PlotStreamKind::Error,
                            v1::PlotEnvelope {
                                plot_id: key,
                                warnings: vec![error.to_string()],
                                ..Default::default()
                            },
                        ),
                    };
                    let frame = v1::PlotStreamFrame {
                        updates: vec![v1::PlotStreamUpdate {
                            kind: kind as i32,
                            envelope: Some(envelope),
                        }],
                    };
                    if sender
                        .send(Ok(v1::PlotsWatchOutput {
                            value: Some(v1::plots_watch_output::Value::Item(frame)),
                        }))
                        .await
                        .is_err()
                    {
                        return Ok::<(), Error>(());
                    }
                }
                if settled {
                    let _ = sender
                        .send(Ok(v1::PlotsWatchOutput {
                            value: Some(v1::plots_watch_output::Value::Response(v1::ClientEmpty {})),
                        }))
                        .await;
                    return Ok(());
                }
                tokio::select! {
                    _=sender.closed()=>return Ok(()),
                    change=changes.changed()=>if change.is_err() {return Err(Error::Invalid("Watch Plots: Reconnect to the Host".into()));}
                }
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Err(error)) => {
                let _ = sender.send(Err(super::status(error))).await;
            }
            Err(_) => {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Plots: Reopen the Watch After Its Deadline")));
            }
            _ => (),
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */

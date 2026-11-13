/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Watch Captured Records

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

pub const ID: &str = "dokime.evidence.watch_records";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::EvidenceWatchRecordsRequest, v1::EvidenceWatchRecordsOutput>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: v1::EvidenceWatchRecordsRequest) -> Result<Output<v1::EvidenceWatchRecordsOutput>> {
    let mut changes = context.host.store().subscribe();
    let first = super::evidence_list_records::handle(
        context,
        v1::EvidenceListRecordsRequest {
            selection: request.selection.clone(),
            after: request.after,
            ..Default::default()
        },
    )
    .await?;
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let mut page = first;
            loop {
                let after = page.scanned.clone();
                let through = (!page.replay_complete).then(|| page.through.clone()).flatten();
                let complete = page.replay_complete;
                let item = v1::EvidenceWatchRecordsOutput {
                    value: Some(v1::evidence_watch_records_output::Value::Item(page)),
                };
                if sender.send(Ok(item)).await.is_err() {
                    return Ok::<(), crate::Error>(());
                }
                let sealed = match request
                    .selection
                    .as_ref()
                    .and_then(|selection| selection.scope.as_ref())
                    .and_then(|scope| scope.execution_id.as_deref())
                {
                    Some(id) => context.host.store().metadata(id)?.state != dokime_storage::ExecutionState::Open,
                    None => false,
                };
                if complete && sealed {
                    let _ = sender
                        .send(Ok(v1::EvidenceWatchRecordsOutput {
                            value: Some(v1::evidence_watch_records_output::Value::Response(v1::ClientEmpty {})),
                        }))
                        .await;
                    return Ok(());
                }
                if complete {
                    tokio::select! {
                        _ = sender.closed() => return Ok(()),
                        changed = changes.changed() => if changed.is_err() { return Err(crate::Error::Invalid("Watch Records: Reconnect to the Host".into())); }
                    }
                }
                page = super::evidence_list_records::handle(
                    &context,
                    v1::EvidenceListRecordsRequest {
                        selection: request.selection.clone(),
                        after,
                        through,
                        ..Default::default()
                    },
                )
                .await?;
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Err(error)) => {
                let _ = sender.send(Err(super::status(error))).await;
            }
            Err(_) => {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Watch Records: Reopen the Watch After Its Deadline")));
            }
            _ => (),
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */

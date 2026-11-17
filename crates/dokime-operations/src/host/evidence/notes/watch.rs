/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{WatchRows, WatchRowsRequest, watch_rows_response::Event};
use harmos_protocol::model::{Failure, OperationContract};
use harmos_runtime::operations::{StreamSettings, native_stream};
use harmos_runtime::storage::ObjectDefinition;
use harmos_runtime::storage::query::{ObjectSelection, ObjectVersions, Source};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use super::list;
use crate::host::{ClientSettings, Context, Output};
use crate::{Error, Result};
use dokime_protocol::host::evidence::{NotesListRequest, NotesWatchRequest, NotesWatchResponse};
use dokime_storage::persisted::Administration;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<NotesWatchRequest, NotesWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: NotesWatchRequest) -> Result<Output<NotesWatchResponse>> {
    crate::host::complete(ID, context.settings.watch_timeout, async {
        let mut selection = NotesListRequest {
            selection: request.selection,
            at: request.at,
            page: request.page,
        };
        let first = list::handle(context, selection.clone()).await?;
        let revision = crate::host::paging::bytes(&selection.selection.clone().unwrap_or_default());
        let at = first.resolved_at.clone().ok_or_else(|| Error::Precondition("Watch Notes: Return the Opening Storage Cut".into()))?;
        let at = list::source_cut(at, &revision)?;
        let schema = ObjectDefinition::of::<Administration>()
            .schema()
            .map_err(|source| Error::Invalid(format!("Watch Notes: Restore Its Registered Object Schema: {source}")))?;
        let limits = StreamSettings {
            items: NonZeroUsize::new(context.settings.stream_buffer).unwrap(),
            bytes: NonZeroUsize::new(context.settings.message_bytes).unwrap(),
        };
        let (send, receive) = native_stream(limits)
            .map_err(|source| Error::Unavailable(format!("Watch Notes: Configure Positive Stream and Message Bounds: {source}")))?;
        let operations = match &context.caller {
            Some(caller) => context
                .operations
                .with_scope(caller.execution.clone())
                .map_err(|source| Error::Permission(format!("Watch Notes: Select the Calling Execution: {source}")))?,
            None => (*context.operations).clone(),
        };
        let invocation = operations
            .submit_native::<WatchRows>(
                WatchRowsRequest {
                    source: Some(Source::Objects(ObjectSelection {
                        schema: Some(schema),
                        key: None,
                        versions: ObjectVersions::Revisions,
                    })),
                    at: Some(at),
                },
                send,
            )
            .map_err(|source| Error::Unavailable(format!("Watch Notes: Reopen From a Fresh Notes List Cut: {source}")))?;
        let context = context.clone();
        let mut runtime = context.host.changes();
        let (sender, output) = mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
                let mut page = first;
                let mut opening = true;
                loop {
                    let cut = page.resolved_at.clone();
                    let token = page.next_page_token;
                    let response = NotesWatchResponse {
                        notes: page.notes,
                        resolved_at: cut.clone(),
                        opening,
                        resync_required: None,
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return Ok(());
                    }
                    if !token.is_empty() {
                        selection.at = cut;
                        selection.page.get_or_insert_default().page_token = token;
                    } else {
                        loop {
                            let Some(frame) = receive.recv().await else { return Ok(()) };
                            match frame.event.as_ref() {
                                Some(Event::DurableProgress(())) => {
                                    // Data Frames Carry the Prior Cut; Recompute Only at the Complete Frontier.
                                    let at = frame
                                        .resolved_at
                                        .clone()
                                        .ok_or_else(|| Error::Precondition("Watch Notes: Return the Complete Storage Frontier".into()))?;
                                    selection.at = Some(list::selection_cut(at, revision.clone()));
                                    break;
                                }
                                Some(Event::ResyncRequired(message)) => {
                                    let _ = sender
                                        .send(Ok(NotesWatchResponse {
                                            resolved_at: frame.resolved_at.clone().map(|at| list::selection_cut(at, revision.clone())),
                                            resync_required: Some(Failure {
                                                code: "resync_required".into(),
                                                message: message.clone(),
                                                ..Default::default()
                                            }),
                                            ..Default::default()
                                        }))
                                        .await;
                                    return Ok(());
                                }
                                _ => continue,
                            }
                        }
                        opening = false;
                        selection.page.get_or_insert_default().page_token.clear();
                    }
                    page = list::handle(&context, selection.clone()).await?;
                }
            };
            let result = tokio::select! {
                _ = sender.closed() => Ok(()),
                _ = async {
                    while !context.host.shutting_down() {
                        if runtime.changed().await.is_err() { break; }
                    }
                } => Ok(()),
                result = tokio::time::timeout(context.settings.watch_timeout, work) => {
                    result.unwrap_or_else(|_| Err(Error::Deadline(ID.into())))
                }
            };
            invocation.request_cancel();
            let settled = invocation.await;
            if let Err(source) = result {
                let _ = sender.send(Err(crate::host::status(source))).await;
            } else if let Err(source) = settled
                && !matches!(source, harmos_runtime::Error::OperationRefused(harmos_runtime::Refusal::Cancelled { .. }))
            {
                let _ = sender.send(Err(Status::internal(format!("{ID}: Reopen From Its Last Issued Cut: {source}")))).await;
            }
        });
        Ok(ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::download!(NotesWatchRequest, NotesWatchResponse, "dokime.host.v1.evidence", "Notes", "Watch");

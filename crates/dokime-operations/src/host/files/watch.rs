/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::{get, read};
use crate::host::{ClientSettings, Context, Output};
use crate::{Error, Result};
use dokime_protocol::host::files_watch_request::Target;
use dokime_protocol::host::{FileBatch, FileCompletion, FilesGetRequest, FilesReadRequest, FilesWatchRequest, FilesWatchResponse};
use harmos_protocol::model::OperationContract;
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesWatchRequest, FilesWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: FilesWatchRequest) -> Result<Output<FilesWatchResponse>> {
    let file = match request.target.clone() {
        Some(Target::File(file)) => file,
        Some(Target::Upload(upload)) => return watch_upload(context, upload, request.offset, request.at).await,
        None => return Err(Error::Invalid(format!("{ID}: Select a File or Upload"))),
    };
    let selected = get::handle(context, FilesGetRequest { file: Some(file), at: request.at }).await?;
    let file = selected.file.ok_or_else(|| Error::Precondition(format!("{ID}: Select Published Content")))?;
    if request.offset > file.length {
        return Err(Error::Invalid(format!("{ID}: Resume Within Its Length")));
    }
    let context = context.clone();
    let (sender, output) = mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let mut offset = request.offset;
            while offset < file.length {
                let response = read::handle(
                    &context,
                    FilesReadRequest {
                        file: Some(file.clone()),
                        offset,
                        max_bytes: 0,
                        at: selected.resolved_at.clone(),
                    },
                )
                .await?;
                if response.data.is_empty() {
                    return Err(Error::Precondition(format!("{ID}: Stored Content Ended Before Its Declared Length")));
                }
                let length = response.data.len() as u64;
                let response = FilesWatchResponse {
                    data: Some(FileBatch { offset, bytes: response.data }),
                    completion: None,
                    resolved_at: response.resolved_at,
                };
                if sender.send(Ok(response)).await.is_err() {
                    return Ok(());
                }
                offset += length;
            }
            let response = FilesWatchResponse {
                data: None,
                completion: Some(FileCompletion { bytes: offset }),
                resolved_at: selected.resolved_at,
            };
            let _ = sender.send(Ok(response)).await;
            Ok(())
        };
        tokio::select! {
            _ = sender.closed() => {},
            result = time::timeout(context.settings.watch_timeout, work) => {
                match result {
                    Ok(Ok(())) => {},
                    Ok(Err(cause)) => { let _ = sender.send(Err(crate::host::status(cause))).await; },
                    Err(_) => { let _ = sender.send(Err(Status::deadline_exceeded(format!("{ID}: Reopen After Its Deadline")))).await; },
                }
            }
        }
    });
    Ok(ReceiverStream::new(output))
}

/* === Private Functions === */

async fn watch_upload(
    context: &Context, upload: harmos_protocol::model::Upload, offset: u64, at: Option<harmos_protocol::model::ReadCut>,
) -> Result<Output<FilesWatchResponse>> {
    use crate::host::files::transfer;
    use harmos_operations::storage::{WatchFileProduction, WatchFileProductionRequest, watch_file_production_response::Event};
    use harmos_runtime::operations::{StreamSettings, native_stream};
    use harmos_runtime::storage::FilePhase;
    use std::num::NonZeroUsize;
    if at.is_some() {
        return Err(Error::Invalid(format!("{ID}: Observe an Upload from Its Live Writer")));
    }
    let file = transfer::staged(&upload)?;
    let limits = StreamSettings {
        items: NonZeroUsize::new(context.settings.stream_buffer).unwrap(),
        bytes: NonZeroUsize::new(context.settings.message_bytes).unwrap(),
    };
    let (send, receive) = native_stream(limits).map_err(|cause| Error::Unavailable(format!("{ID}: {cause}")))?;
    let invocation = context
        .operations
        .submit_native::<WatchFileProduction>(WatchFileProductionRequest { file: Some(file), offset }, send)
        .map_err(|cause| Error::Unavailable(format!("{ID}: {cause}")))?;
    let (sender, output) = mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            while let Some(value) = receive.recv().await {
                let response = match value.event.clone() {
                    Some(Event::Data(batch)) => FilesWatchResponse {
                        data: Some(FileBatch { offset: batch.offset, bytes: batch.bytes }),
                        completion: None,
                        resolved_at: None,
                    },
                    Some(Event::Progress(progress)) if progress.phase == FilePhase::Finished => FilesWatchResponse {
                        data: None,
                        completion: Some(FileCompletion { bytes: progress.length }),
                        resolved_at: None,
                    },
                    Some(Event::Progress(progress)) if progress.phase == FilePhase::Abandoned => {
                        return Err(Error::Precondition(format!("{ID}: Upload Was Abandoned")));
                    }
                    _ => continue,
                };
                if sender.send(Ok(response)).await.is_err() {
                    break;
                }
            }
            Ok(())
        };
        let result = tokio::select! {
            _ = sender.closed() => Ok(()),
            result = work => result,
        };
        invocation.request_cancel();
        let settled = invocation.await;
        if let Err(cause) = result {
            let _ = sender.send(Err(crate::host::status(cause))).await;
        } else if let Err(cause) = settled
            && !matches!(cause, harmos_runtime::Error::OperationRefused(harmos_runtime::Refusal::Cancelled { .. }))
        {
            let _ = sender.send(Err(Status::internal(format!("{ID}: {cause}")))).await;
        }
    });
    Ok(ReceiverStream::new(output))
}

/* === Tests === */

crate::native::download!(FilesWatchRequest, FilesWatchResponse, "dokime.host.v1", "Files", "Watch");

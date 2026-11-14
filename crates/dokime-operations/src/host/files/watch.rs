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

use crate::Result;
use crate::host::{ClientSettings, Context, Output};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesWatchRequest;
use dokime_protocol::host::FilesWatchResponse;
use dokime_protocol::host::content::ExecutionFileState;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::{FileBatch, FileCompletion};
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
    let mut storage = context.host.store().subscribe();
    let mut runtime = context.host.changes();
    let first = crate::host::files::get::handle(
        context,
        FilesGetRequest {
            file: request.file.clone(),
            at: request.at.clone(),
        },
    )
    .await?;
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let mut offset = request.offset;
            let mut selected = first;
            loop {
                let file = selected.file.as_ref().expect("file metadata response contains its reference");
                if offset > file.length {
                    return Err(crate::Error::Invalid("Watch File: Resume Within Its Length".into()));
                }
                while offset < file.length {
                    let bytes =
                        crate::host::files::transfer::read(&context, file, offset, crate::host::files::transfer::chunk_bytes(&context))?;
                    let size = bytes.len() as u64;
                    let response = FilesWatchResponse {
                        data: Some(FileBatch { offset, bytes }),
                        completion: None,
                        resolved_at: selected.resolved_at.clone(),
                    };
                    if sender.send(Ok(response)).await.is_err() {
                        return Ok(());
                    }
                    offset += size;
                }
                let id = crate::host::files::transfer::identifier(Some(file))?;
                let current = if context.host.temporary_report(id)?.is_some() { None } else { Some(context.host.store().content(id)?) };
                if current.as_ref().is_some_and(|entry| entry.phase == ExecutionFileState::Interrupted) {
                    return Err(crate::Error::Cancelled("Watch File: Production Was Abandoned; Select Published Content".into()));
                }
                // Drain a Final Append Before Sending Completion.
                if current.as_ref().is_some_and(|entry| entry.length > offset) {
                    selected = crate::host::files::get::handle(&context, FilesGetRequest { file: request.file.clone(), at: None }).await?;
                    continue;
                }
                let finished = current.is_none_or(|entry| entry.phase != ExecutionFileState::Streaming);
                if finished {
                    let response = FilesWatchResponse {
                        data: None,
                        completion: Some(FileCompletion { bytes: offset }),
                        resolved_at: selected.resolved_at,
                    };
                    let _ = sender.send(Ok(response)).await;
                    return Ok(());
                }
                if context.host.shutting_down() {
                    return Ok(());
                }
                tokio::select! { _ = sender.closed() => return Ok(()), changed = storage.changed() => if changed.is_err() { return Ok(()); }, changed = runtime.changed() => if changed.is_err() { return Ok(()); } }
                selected = crate::host::files::get::handle(&context, FilesGetRequest { file: request.file.clone(), at: None }).await?;
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = sender.send(Err(crate::host::status(error))).await;
            }
            Err(_) => {
                let _ = sender.send(Err(Status::deadline_exceeded("Watch File: Reopen After Its Deadline"))).await;
            }
        }
    });
    Ok(ReceiverStream::new(output))
}

/* === Private Functions === */

/* === Tests === */

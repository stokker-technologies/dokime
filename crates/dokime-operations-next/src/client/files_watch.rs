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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::content::ExecutionFileState;
use dokime_protocol::client::{FilesGetRequest, FilesWatchRequest, FilesWatchResponse};
use harmos_protocol::storage::{FileBatch, FileCompletion};
use harmos_protocol::v1::OperationContract;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.Files/Watch";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<FilesWatchRequest, FilesWatchResponse>(ID, settings.watch_timeout)
}
pub async fn handle(context: &Context, request: FilesWatchRequest) -> Result<super::Output<FilesWatchResponse>> {
    let mut storage = context.host.store().subscribe();
    let first = super::files_get::handle(
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
                    let bytes = super::files_transfer::read(&context, file, offset, super::files_transfer::chunk_bytes(&context))?;
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
                let id = super::files_transfer::identifier(Some(file))?;
                let finished = context.host.temporary_report(id)?.is_some()
                    || context.host.store().content(id)?.phase != ExecutionFileState::Streaming;
                if finished {
                    let response = FilesWatchResponse {
                        data: None,
                        completion: Some(FileCompletion { bytes: offset }),
                        resolved_at: selected.resolved_at,
                    };
                    let _ = sender.send(Ok(response)).await;
                    return Ok(());
                }
                tokio::select! { _ = sender.closed() => return Ok(()), changed = storage.changed() => if changed.is_err() { return Ok(()); } }
                selected = super::files_get::handle(&context, FilesGetRequest { file: request.file.clone(), at: None }).await?;
            }
        };
        match tokio::time::timeout(context.settings.watch_timeout, work).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = sender.send(Err(super::status(error))).await;
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

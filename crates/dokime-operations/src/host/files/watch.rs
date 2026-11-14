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
    let selected = get::handle(
        context,
        FilesGetRequest {
            file: request.file.clone(),
            at: request.at,
        },
    )
    .await?;
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

/* === Tests === */

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod codec;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use futures_util::stream;
use sha2::{Digest, Sha256};
use tokio::sync::watch;
use tonic::Status;
// Local Imports
use super::super::chunked;
use super::super::error::detailed;
use dokime_app::{Error, ExecutionFileStream, RuntimeHandle};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type DownloadStream = chunked::FrameStream<super::super::charged::Bytes>;

/* === Structs === */
struct Transfer {
    source: ExecutionFileStream,
    digest: Sha256,
    hashed: u64,
    maximum: NonZeroUsize,
    projection: NonZeroUsize,
    message_limit: usize,
    stopping: watch::Receiver<bool>,
    phase: Phase,
}

/* === Enums === */
enum Phase {
    Metadata,
    Chunks,
    Done,
}

/* === Traits === */

/* === Implementations === */
impl Transfer {
    async fn next(&mut self) -> Result<Option<super::super::charged::Bytes>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable(
                "execution transfer interrupted by listener shutdown",
            ));
        }
        let projected = match self.phase {
            Phase::Metadata => {
                let frame = self
                    .source
                    .project_metadata(self.projection, |metadata, allowance| {
                        codec::metadata(metadata, allowance, self.message_limit)
                    })
                    .map_err(stream_failure)?;
                self.phase = Phase::Chunks;
                frame
            }
            Phase::Chunks => {
                let digest = &mut self.digest;
                let hashed = &mut self.hashed;
                let message_limit = self.message_limit;
                let pending = self.source.next_chunk_projected(
                    self.maximum,
                    self.projection,
                    |content, allowance| {
                        let end = content
                            .offset
                            .checked_add(content.bytes.len() as u64)
                            .ok_or(harmos::Error::InvalidInput {
                                field: "execution download",
                                reason: "chunk offset overflows",
                            })?;
                        if content.offset != *hashed && end != *hashed {
                            return Err(harmos::Error::InvalidInput {
                                field: "execution download",
                                reason: "digest prefix differs from original chunk",
                            });
                        }
                        let frame = codec::chunk(content, allowance, message_limit)?;
                        // A Timeout After This Callback May Retain The Original Chunk For Retry.
                        if content.offset == *hashed {
                            digest.update(&content.bytes);
                            *hashed = end;
                        }
                        Ok(frame)
                    },
                );
                let frame = tokio::select! { biased;
                    _ = self.stopping.changed() => return Err(Status::unavailable("execution transfer interrupted by listener shutdown")),
                    frame = pending => frame.map_err(stream_failure)?,
                };
                if let Some(frame) = frame {
                    return Ok(Some(frame));
                }
                if self.hashed != self.source.metadata().size_bytes {
                    return Err(Status::data_loss(
                        "execution output ended before declared prefix",
                    ));
                }
                let frame = self
                    .source
                    .project_metadata(self.projection, |metadata, allowance| {
                        codec::complete(
                            metadata.size_bytes,
                            &self.digest,
                            allowance,
                            self.message_limit,
                        )
                    })
                    .map_err(stream_failure)?;
                self.phase = Phase::Done;
                frame
            }
            Phase::Done => return Ok(None),
        };
        Ok(Some(projected))
    }
}

/* === Public Functions === */
pub(super) async fn download(
    runtime: &RuntimeHandle,
    request: v1::DownloadExecutionFileRequest,
    max_message_bytes: usize,
    stopping: watch::Receiver<bool>,
) -> Result<DownloadStream, Status> {
    if *stopping.borrow() {
        return Err(Status::unavailable("listener is stopping"));
    }
    let (id, _) = super::files::execution(runtime, &request.execution_id)?;
    super::super::value::text(&request.relative_path)?;
    if request.relative_path.is_empty() {
        return Err(Status::invalid_argument("output name is required"));
    }
    let content = runtime
        .open_execution_file_stream(&id, request.relative_path)
        .await
        .map_err(stream_failure)?
        .ok_or_else(|| Status::not_found("registered execution output not found"))?;
    if content.metadata().size_bytes > super::MAX_CONTENT_BYTES {
        return Err(Status::resource_exhausted(
            "execution output exceeds native content limit",
        ));
    }
    let chunk_bytes = super::transfer_chunk_bytes(max_message_bytes);
    let maximum = NonZeroUsize::new(chunk_bytes).ok_or_else(|| {
        Status::resource_exhausted("message limit cannot contain execution chunk")
    })?;
    let projection = max_message_bytes
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| Status::resource_exhausted("execution frame memory overflows"))?;
    let transfer = Transfer {
        source: content,
        digest: Sha256::new(),
        hashed: 0,
        maximum,
        projection,
        message_limit: max_message_bytes,
        stopping,
        phase: Phase::Metadata,
    };
    Ok(Box::pin(stream::unfold(
        Some(transfer),
        |state| async move {
            let mut transfer = state?;
            match transfer.next().await {
                Ok(Some(frame)) => Some((Ok(frame), Some(transfer))),
                Ok(None) => None,
                Err(error) => Some((Err(error), None)),
            }
        },
    )))
}

/* === Private Functions === */
fn stream_failure(error: Error) -> Status {
    let status = match &error {
        Error::Harmos(harmos::Error::ResourceExhausted { resource }) => Status::resource_exhausted(
            format!("execution output exceeds its original {resource} budget"),
        ),
        Error::ExecutionFileTooLarge => {
            Status::resource_exhausted("execution output exceeds native content limit")
        }
        Error::ShuttingDown | Error::Interrupted => {
            Status::unavailable("execution output capture interrupted")
        }
        _ => Status::internal("execution output could not be captured"),
    };

    detailed(status, &error)
}

/* === Tests === */

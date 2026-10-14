/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod frames;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response, StreamOutput};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::{error::detailed, value};
use dokime_app::{Error, RuntimeHandle};
use dokime_contract::operations::content as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Download {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Download {
    type Declaration = wire::DownloadExecutionFile;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::ExecutionFileDownloadFrame>;
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::DownloadExecutionFileRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let runtime = &self.resources.runtime;
        let message_limit = self.resources.maximum;
        let (id, _) = super::files::execution(runtime, &request.execution_id)?;
        value::text(&request.relative_path)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        let mut source = runtime
            .open_execution_file_stream(&id, std::mem::take(&mut request.relative_path))
            .await
            .map_err(stream_failure)?
            .ok_or_else(|| Status::not_found("registered execution output not found"))?;
        if source.metadata().size_bytes > super::MAX_CONTENT_BYTES {
            return Err(Status::resource_exhausted(
                "execution output exceeds native content limit",
            ));
        }
        let maximum =
            NonZeroUsize::new(super::transfer_chunk_bytes(message_limit)).ok_or_else(|| {
                Status::resource_exhausted("message limit cannot contain execution chunk")
            })?;
        let metadata_heap = source
            .metadata()
            .relative_path
            .len()
            .checked_add(
                source
                    .metadata()
                    .content_type
                    .as_ref()
                    .map_or(0, String::len),
            )
            .ok_or_else(|| Status::resource_exhausted("execution metadata memory overflows"))?;
        let projection = maximum
            .get()
            .max(metadata_heap)
            .max(64)
            .checked_add(std::mem::size_of::<wire::ExecutionFileDownloadFrame>())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| Status::resource_exhausted("execution frame memory overflows"))?;
        let metadata = source
            .project_metadata(projection, |metadata, allowance| {
                frames::metadata(metadata, allowance, message_limit)
            })
            .map_err(stream_failure)?;
        let owner = metadata
            .try_into_response()
            .map_err(|error| stream_failure(error.into()))?;
        output
            .send_owned(owner)
            .await
            .map_err(|error| stream_failure(error.into()))?;
        let mut digest = Sha256::new();
        let mut hashed = 0u64;
        loop {
            let frame = source
                .next_chunk_projected(maximum, projection, |content, allowance| {
                    let end = content
                        .offset
                        .checked_add(content.bytes.len() as u64)
                        .ok_or(harmos::Error::InvalidInput {
                            field: "execution download",
                            reason: "chunk offset overflows",
                        })?;
                    if content.offset != hashed && end != hashed {
                        return Err(harmos::Error::InvalidInput {
                            field: "execution download",
                            reason: "digest prefix differs from original chunk",
                        });
                    }
                    let frame = frames::chunk(content, allowance, message_limit)?;
                    // A Timeout May Retain The Original Chunk For Retry.
                    if content.offset == hashed {
                        digest.update(&content.bytes);
                        hashed = end;
                    }
                    Ok(frame)
                })
                .await
                .map_err(stream_failure)?;
            let Some(frame) = frame else {
                break;
            };
            let owner = frame
                .try_into_response()
                .map_err(|error| stream_failure(error.into()))?;
            output
                .send_owned(owner)
                .await
                .map_err(|error| stream_failure(error.into()))?;
        }
        if hashed != source.metadata().size_bytes {
            return Err(Status::data_loss(
                "execution output ended before declared prefix",
            ));
        }
        let complete = source
            .project_metadata(projection, |metadata, allowance| {
                frames::complete(metadata.size_bytes, &digest, allowance, message_limit)
            })
            .map_err(stream_failure)?;
        let owner = complete
            .try_into_response()
            .map_err(|error| stream_failure(error.into()))?;
        output
            .send_owned(owner)
            .await
            .map_err(|error| stream_failure(error.into()))?;
        Ok(().into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Download>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
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

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
use dokime::harmos::{Context, RegistrationExt};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::error::detailed;
use dokime::operations::content as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::{RuntimeHandle, runtime::Error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::DownloadExecutionFile> for Resources {
    fn serve<'a>(&'a self, request: wire::DownloadExecutionFile, ctx: Context<'a, wire::DownloadExecutionFile>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let runtime = &self.runtime;
                let message_limit = self.maximum;
                let (id, _) = super::files::execution(runtime, &request.execution_id)?;
                if request.relative_path.is_empty() {
                    return Err(Status::invalid_argument("output name is required"));
                }
                let mut source = runtime
                    .open_execution_file_stream(&id, request.relative_path)
                    .await
                    .map_err(stream_failure)?
                    .ok_or_else(|| Status::not_found("registered execution output not found"))?;
                if source.metadata().size_bytes > 2 * 1024 * 1024 * 1024 {
                    return Err(Status::resource_exhausted("execution output exceeds the transfer size limit"));
                }
                let cap = super::download_chunk_bytes(runtime, message_limit)?;
                let template = dokime::domain::model::content::wire::ExecutionFileDownloadFrame {
                    payload: Some(dokime::domain::model::content::wire::ExecutionFileDownloadFramePayload::Chunk(
                        dokime::domain::model::content::wire::ArtifactChunk {
                            offset: source.metadata().size_bytes,
                            data: Vec::new(),
                        },
                    )),
                };
                let bytes = super::output_chunk_bytes(&template, cap, message_limit)?;
                let maximum =
                    NonZeroUsize::new(bytes).ok_or_else(|| Status::resource_exhausted("message limit cannot contain execution chunk"))?;
                let metadata = frames::metadata(source.metadata(), message_limit).map_err(|error| stream_failure(error.into()))?;
                ctx.streams().send_owned(metadata).await.map_err(|error| stream_failure(error.into()))?;
                let mut digest = Sha256::new();
                let mut hashed = 0u64;
                loop {
                    let Some(content) = source.next_chunk(maximum).await.map_err(stream_failure)? else {
                        break;
                    };
                    if content.offset != hashed {
                        return Err(Status::data_loss("execution output chunk does not continue its prefix"));
                    }
                    digest.update(&content.bytes);
                    hashed = content.offset + content.bytes.len() as u64;
                    let frame = frames::chunk(content, message_limit).map_err(|error| stream_failure(error.into()))?;
                    ctx.streams().send_owned(frame).await.map_err(|error| stream_failure(error.into()))?;
                }
                if hashed != source.metadata().size_bytes {
                    return Err(Status::data_loss("execution output ended before declared prefix"));
                }
                let complete =
                    frames::complete(source.metadata().size_bytes, &digest, message_limit).map_err(|error| stream_failure(error.into()))?;
                ctx.streams().send_owned(complete).await.map_err(|error| stream_failure(error.into()))?;
                Ok(().into())
            })
            .await
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::DownloadExecutionFile>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::DownloadExecutionFile>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

fn stream_failure(error: Error) -> Status {
    let status = match &error {
        Error::Harmos(dokime::harmos::Error::ResourceExhausted { resource }) => {
            Status::resource_exhausted(format!("execution output exceeds its original {resource} budget"))
        }
        Error::ShuttingDown | Error::Interrupted => Status::unavailable("execution output capture interrupted"),
        _ => Status::internal("execution output could not be captured"),
    };

    detailed(status, &error)
}

/* === Tests === */

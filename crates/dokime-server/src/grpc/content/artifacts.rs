/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos::Protobuf;
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use crate::grpc::{
    chunked::{self, ArtifactFrames},
    error::detailed,
    value,
};
use dokime_app::{RuntimeHandle, core::ArtifactId, runtime::ArtifactReader};
use dokime_contract::operations::content as content_wire;

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
    type Declaration = content_wire::DownloadArtifact;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<content_wire::ArtifactFrame>;
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut content_wire::DownloadArtifactRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        value::text(&request.artifact_id)?;
        let id = ArtifactId::from_uuid(request.artifact_id.clone())
            .map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
        let reader = self
            .resources
            .runtime
            .open_artifact_stream(&id)
            .await
            .map_err(|source| {
                let status = if matches!(
                    &source,
                    dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
                ) {
                    Status::resource_exhausted("cannot open stored artifact")
                } else {
                    Status::internal("cannot open stored artifact")
                };
                detailed(status, &source)
            })?
            .ok_or_else(|| Status::not_found("artifact not found"))?;
        deliver(
            reader,
            self.resources.maximum,
            super::transfer_chunk_bytes(self.resources.maximum),
            output,
            metadata_failure,
        )
        .await?;
        Ok(().into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    super::list::register(runtime, maximum)?;
    super::inventory::register(runtime, maximum)?;
    super::files::register(runtime, maximum)?;
    super::execution::register(runtime, maximum)?;
    super::follow::register(runtime, maximum)?;
    super::upload::register(runtime, maximum)?;
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

pub(in crate::grpc) async fn deliver(
    reader: ArtifactReader,
    maximum: usize,
    chunk_bytes: usize,
    output: &mut StreamOutput<content_wire::ArtifactFrame>,
    failure: fn(dokime_app::Error) -> Status,
) -> Result<(), Status> {
    let required = std::mem::size_of::<content_wire::ArtifactFrame>()
        .saturating_add(super::metadata_memory(reader.record())?);
    if required > maximum {
        return Err(Status::resource_exhausted(
            "artifact metadata exceeds projection budget",
        ));
    }
    let allowance = NonZeroUsize::new(required)
        .ok_or_else(|| Status::resource_exhausted("invalid artifact metadata allowance"))?;
    let projected = reader
        .project_record(allowance, |record, _| {
            let frame = content_wire::ArtifactFrame {
                payload: Some(content_wire::ArtifactFramePayload::Metadata(
                    super::metadata(record).map_err(projection_failure)?,
                )),
            };
            if frame.encoded_len() > maximum {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "artifact metadata encoding",
                });
            }
            Ok(frame)
        })
        .map_err(failure)?;
    let owner = projected
        .try_into_response()
        .map_err(|source| failure(source.into()))?;
    let size_bytes = reader.record().size_bytes;
    let sha256 = reader.record().sha256.clone();
    chunked::Download {
        size_bytes,
        sha256,
        frames: ArtifactFrames(owner),
        source: chunked::Source::artifact(reader),
        noun: "artifact",
        read_failure: "artifact content could not be verified",
        chunk_bytes,
        message_limit: maximum,
    }
    .run(output)
    .await
}

/* === Private Functions === */
fn metadata_failure(source: dokime_app::Error) -> Status {
    let status = if matches!(
        &source,
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
    ) {
        Status::resource_exhausted("artifact metadata exceeds its budget")
    } else {
        Status::data_loss("invalid stored artifact metadata")
    };
    detailed(status, &source)
}
fn projection_failure(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return harmos::Error::ResourceExhausted {
            resource: "artifact metadata projection",
        };
    }
    harmos::Error::backend(
        "project captured artifact metadata",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */

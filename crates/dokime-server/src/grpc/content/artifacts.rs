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
use dokime_runtime::RuntimeHandle;
use dokime_domain::model::identity::ArtifactId;
use dokime_runtime::runtime::ArtifactReader;
use dokime_domain::operations::content as content_wire;

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
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut content_wire::DownloadArtifact,
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
                    dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })
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
        dokime_runtime::app::Lanes::Open,
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
) -> Result<(), Status> {
    let frame = content_wire::ArtifactFrame {
        payload: Some(content_wire::ArtifactFramePayload::Metadata(
            super::metadata(reader.record())?,
        )),
    };
    if frame.encoded_len() > maximum {
        return Err(Status::resource_exhausted(
            "artifact metadata exceeds the message limit",
        ));
    }
    let size_bytes = reader.record().size_bytes;
    let sha256 = reader.record().sha256.clone();
    chunked::Download {
        size_bytes,
        sha256,
        frames: ArtifactFrames(frame.into()),
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

/* === Tests === */

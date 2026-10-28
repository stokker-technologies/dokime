/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::operations::StreamOutput;
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

use crate::grpc::{
    chunked::{self, ArtifactFrames},
    error::detailed,
};
use dokime::RuntimeHandle;
use dokime::domain::model::identity::ArtifactId;
use dokime::operations::content as content_wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::ArtifactReader;

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

impl Serve<content_wire::DownloadArtifact> for Resources {
    fn serve<'a>(&'a self, request: content_wire::DownloadArtifact, ctx: Context<'a, content_wire::DownloadArtifact>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let id =
                    ArtifactId::from_uuid(request.artifact_id.clone()).map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
                let reader = self
                    .runtime
                    .open_artifact_stream(&id)
                    .await
                    .map_err(|source| {
                        let status = if matches!(&source, dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. })) {
                            Status::resource_exhausted("cannot open stored artifact")
                        } else {
                            Status::internal("cannot open stored artifact")
                        };
                        detailed(status, &source)
                    })?
                    .ok_or_else(|| Status::not_found("artifact not found"))?;
                deliver(reader, self.maximum, super::download_chunk_bytes(&self.runtime, self.maximum)?, ctx.streams()).await?;
                Ok(().into())
            })
            .await
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    super::list::register(runtime, maximum).await?;
    super::inventory::register(runtime, maximum).await?;
    super::files::register(runtime, maximum).await?;
    super::execution::register(runtime, maximum).await?;
    super::follow::register(runtime, maximum).await?;
    super::upload::register(runtime, maximum).await?;
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<content_wire::DownloadArtifact>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<content_wire::DownloadArtifact>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(in crate::grpc) async fn deliver(
    reader: ArtifactReader, maximum: usize, chunk_bytes: usize, output: &StreamOutput<dokime::domain::model::content::wire::ArtifactFrame>,
) -> Result<(), Status> {
    let frame = dokime::domain::model::content::wire::ArtifactFrame {
        payload: Some(dokime::domain::model::content::wire::ArtifactFramePayload::Metadata(super::metadata(reader.record())?)),
    };
    if super::output_frame_len(&frame)? > maximum {
        return Err(Status::resource_exhausted("artifact metadata exceeds the message limit"));
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

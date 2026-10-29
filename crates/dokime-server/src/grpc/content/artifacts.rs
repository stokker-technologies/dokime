/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::Context;
use dokime::harmos::operations::StreamOutput;
use tonic::Status;

use crate::grpc::{
    chunked::{self, ArtifactFrames},
    error::detailed,
};
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::ArtifactId;
use dokime::operations::content as content_wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::ArtifactReader;

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(content_wire::DownloadArtifact),
}

impl Serve<content_wire::DownloadArtifact> for Resources {
    fn serve<'a>(&'a self, request: content_wire::DownloadArtifact, ctx: Context<'a, content_wire::DownloadArtifact>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let id =
                    ArtifactId::from_uuid(request.artifact_id.clone()).map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
                let reader = dokime::app::live_runtime(&self.runtime)
                    .await?
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
                deliver(
                    reader,
                    self.maximum,
                    super::download_chunk_bytes(&dokime::app::live_runtime(&self.runtime).await?, self.maximum)?,
                    ctx.streams(),
                )
                .await?;
                Ok(().into())
            })
            .await
        })
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    super::list::prepare(setup, runtime, maximum)?;
    super::inventory::prepare(setup, runtime, maximum)?;
    super::files::prepare(setup, runtime, maximum)?;
    super::execution::prepare(setup, runtime, maximum)?;
    super::follow::prepare(setup, runtime, maximum)?;
    super::upload::prepare(setup, runtime, maximum)?;
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<content_wire::DownloadArtifact>>(Host::<content_wire::DownloadArtifact>::new(served));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
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

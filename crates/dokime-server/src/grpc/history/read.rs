/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

use super::super::content;
use super::wire;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::ArtifactId;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

impl Serve<history::DownloadCheckpointArtifact> for Resources {
    fn serve<'a>(
        &'a self, mut request: history::DownloadCheckpointArtifact, ctx: Context<'a, history::DownloadCheckpointArtifact>,
    ) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let checkpoint = wire::checkpoint(request.checkpoint.take())?;
                let artifact_id =
                    ArtifactId::from_uuid(request.artifact_id.clone()).map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
                let reader = self.runtime.checkpoint_artifact(checkpoint, &artifact_id).await.map_err(wire::checkpoint_failure)?;
                content::artifacts::deliver(reader, self.maximum, content::download_chunk_bytes(&self.runtime, self.maximum)?, output)
                    .await?;
                Ok(().into())
            })
            .await
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<history::DownloadCheckpointArtifact>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<history::DownloadCheckpointArtifact>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

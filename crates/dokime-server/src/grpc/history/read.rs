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
use tonic::Status;

use super::super::content;
use super::wire;
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::ArtifactId;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(history::DownloadCheckpointArtifact),
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
                let reader = dokime::app::live_runtime(&self.runtime)
                    .await?
                    .checkpoint_artifact(checkpoint, &artifact_id)
                    .await
                    .map_err(wire::checkpoint_failure)?;
                content::artifacts::deliver(
                    reader,
                    self.maximum,
                    content::download_chunk_bytes(&dokime::app::live_runtime(&self.runtime).await?, self.maximum)?,
                    output,
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
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<history::DownloadCheckpointArtifact>>(Host::<history::DownloadCheckpointArtifact>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

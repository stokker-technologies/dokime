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
use dokime::harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use super::super::content;
use super::wire;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::ArtifactId;
use dokime::domain::operations::history;

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
    type Declaration = history::DownloadCheckpointArtifact;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::content::wire::ArtifactFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut history::DownloadCheckpointArtifact, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let artifact_id =
            ArtifactId::from_uuid(request.artifact_id.clone()).map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
        let reader = self.resources.runtime.checkpoint_artifact(checkpoint, &artifact_id).await.map_err(wire::checkpoint_failure)?;
        content::artifacts::deliver(
            reader,
            self.resources.maximum,
            content::download_chunk_bytes(&self.resources.runtime, self.resources.maximum)?,
            output,
        )
        .await?;
        Ok(().into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Download>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

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
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use super::super::{content, value};
use super::wire;
use dokime_runtime::RuntimeHandle;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::operations::content as content_wire;
use dokime_domain::operations::history;

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
        request: &mut history::DownloadCheckpointArtifactRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        value::text(&request.artifact_id)?;
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let artifact_id = ArtifactId::from_uuid(request.artifact_id.clone())
            .map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
        let reader = self
            .resources
            .runtime
            .checkpoint_artifact(checkpoint, &artifact_id)
            .await
            .map_err(wire::checkpoint_failure)?;
        content::artifacts::deliver(
            reader,
            self.resources.maximum,
            (64 * 1024).min(self.resources.maximum.saturating_sub(1024)),
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
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
/* === Tests === */

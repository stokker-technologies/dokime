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
use std::{num::NonZeroUsize, sync::Arc, time::Instant};
// Third Party Imports
use harmos::{
    Protobuf,
    operations::{Context, Operation, Response, StreamOutput},
};
use tonic::Status;
// Local Imports
use crate::grpc::{error::detailed, value};
use dokime_runtime::{runtime::Error, RuntimeHandle};
use dokime_domain::operations::content as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Follow {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Follow {
    type Declaration = wire::FollowExecutionFile;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::ExecutionFileFollowFrame>;
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::FollowExecutionFile,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let runtime = &self.resources.runtime;
        let (execution, _) = super::files::execution(runtime, &request.execution_id)?;
        value::text(&request.relative_path)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        if request.max_bytes > 64 * 1024 {
            return Err(Status::invalid_argument("follow chunk exceeds 64 KiB"));
        }
        if request.offset != 0 && request.identity.is_none() {
            return Err(Status::invalid_argument(
                "resume offset requires original identity",
            ));
        }
        if let Some(identity) = &request.identity {
            value::text(&identity.file_id)?;
            value::text(&identity.execution_id)?;
            value::text(&identity.relative_path)?;
            let history = identity
                .history
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("original history is required"))?;
            value::text(&history.store)?;
            value::text(&history.epoch)?;
        }
        let requested = if request.max_bytes == 0 {
            64 * 1024
        } else {
            request.max_bytes as usize
        };
        let maximum = NonZeroUsize::new(
            requested.min(super::transfer_chunk_bytes(self.resources.maximum)),
        )
        .ok_or_else(|| Status::resource_exhausted("listener cannot contain a follow chunk"))?;
        let mut follower = runtime
            .follow_execution_file(
                &execution,
                &request.relative_path,
                request.identity.as_ref(),
                request.offset,
            )
            .await
            .map_err(failure)?
            .ok_or_else(|| Status::not_found("registered execution output not found"))?;
        let started = Instant::now();
        tracing::debug!(execution_id = %execution, offset = request.offset, "execution file follow admitted");
        let metadata = follower.metadata_frame();
        send(output, metadata, self.resources.maximum).await?;
        while let Some(frame) = follower.next_frame(maximum).await.map_err(failure)? {
            let terminal = matches!(
                &frame.payload,
                Some(wire::ExecutionFileFollowFramePayload::Terminal(_))
            );
            send(output, frame, self.resources.maximum).await?;
            if terminal {
                tracing::debug!(execution_id = %execution,
                    elapsed_ms = started.elapsed().as_millis() as u64, "execution file follow terminal sent");
            }
        }
        Ok(().into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Follow>(
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
async fn send(
    output: &mut StreamOutput<wire::ExecutionFileFollowFrame>,
    frame: wire::ExecutionFileFollowFrame,
    maximum: usize,
) -> Result<(), Status> {
    if frame.encoded_len() > maximum {
        return Err(Status::resource_exhausted(
            "follow frame exceeds listener message limit",
        ));
    }
    output
        .send_owned(frame.into())
        .await
        .map_err(|error| failure(error.into()))
}
fn failure(error: Error) -> Status {
    let status = match &error {
        Error::Harmos(harmos::Error::InvalidInput { reason, .. }) => {
            Status::failed_precondition(*reason)
        }
        Error::Harmos(harmos::Error::ResourceExhausted { resource }) => {
            Status::resource_exhausted(format!("execution file exceeds original {resource} budget"))
        }
        Error::ExecutionFileTooLarge => {
            Status::resource_exhausted("execution output exceeds native content limit")
        }
        Error::Operation(reason) => Status::failed_precondition(reason),
        Error::ShuttingDown | Error::Interrupted => {
            Status::unavailable("file following interrupted")
        }
        _ => Status::internal("execution output could not be followed"),
    };
    detailed(status, &error)
}

/* === Tests === */

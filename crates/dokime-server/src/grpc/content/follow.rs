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
use dokime::harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use crate::grpc::error::detailed;
use dokime::domain::operations::content as wire;
use dokime::{RuntimeHandle, runtime::Error};

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
    type Endpoints = StreamOutput<dokime::domain::model::content::wire::ExecutionFileFollowFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::FollowExecutionFile, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let runtime = &self.resources.runtime;
        let (execution, _) = super::files::execution(runtime, &request.execution_id)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        if request.offset != 0 && request.identity.is_none() {
            return Err(Status::invalid_argument("resume offset requires original identity"));
        }
        if let Some(identity) = &request.identity {
            identity.history.as_ref().ok_or_else(|| Status::invalid_argument("original history is required"))?;
        }
        let cap = super::download_chunk_bytes(runtime, self.resources.maximum)?;
        let template = dokime::domain::model::content::wire::ExecutionFileFollowFrame {
            payload: Some(dokime::domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(
                dokime::domain::model::content::wire::ArtifactChunk { offset: u64::MAX, data: Vec::new() },
            )),
        };
        let transport_maximum = super::output_chunk_bytes(&template, cap, self.resources.maximum)?;
        let requested = if request.max_bytes == 0 { transport_maximum } else { request.max_bytes as usize };
        let maximum = NonZeroUsize::new(requested.min(transport_maximum))
            .ok_or_else(|| Status::resource_exhausted("listener cannot contain a follow chunk"))?;
        let mut follower = runtime
            .follow_execution_file(&execution, &request.relative_path, request.identity.as_ref(), request.offset)
            .await
            .map_err(failure)?
            .ok_or_else(|| Status::not_found("registered execution output not found"))?;
        let started = Instant::now();
        tracing::debug!(execution_id = %execution, offset = request.offset, "execution file follow admitted");
        let metadata = follower.metadata_frame().map_err(failure)?;
        send(output, metadata, self.resources.maximum).await?;
        while let Some(frame) = follower.next_frame(maximum).await.map_err(failure)? {
            let terminal =
                matches!(&frame.payload, Some(dokime::domain::model::content::wire::ExecutionFileFollowFramePayload::Terminal(_)));
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
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

async fn send(
    output: &mut StreamOutput<dokime::domain::model::content::wire::ExecutionFileFollowFrame>,
    frame: Response<dokime::domain::model::content::wire::ExecutionFileFollowFrame>, maximum: usize,
) -> Result<(), Status> {
    if super::output_frame_len(&*frame)? > maximum {
        return Err(Status::resource_exhausted("follow frame exceeds listener message limit"));
    }
    let sent = output.send_owned(frame).await;
    sent.map_err(|error| {
        let source = error.into();
        failure(source)
    })
}

fn failure(error: Error) -> Status {
    let status = match &error {
        Error::Harmos(dokime::harmos::Error::InvalidInput { reason, .. }) => Status::failed_precondition(*reason),
        Error::Harmos(dokime::harmos::Error::ResourceExhausted { resource }) => {
            Status::resource_exhausted(format!("execution file exceeds original {resource} budget"))
        }
        Error::Operation(reason) => Status::failed_precondition(reason),
        Error::ShuttingDown | Error::Interrupted => Status::unavailable("file following interrupted"),
        _ => Status::internal("execution output could not be followed"),
    };
    detailed(status, &error)
}

/* === Tests === */

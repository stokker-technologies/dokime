/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{num::NonZeroUsize, sync::Arc, time::Instant};

use dokime::harmos::Context;
use dokime::harmos::operations::{Response, StreamOutput};
use tonic::Status;

use crate::grpc::error::detailed;
use dokime::app::LiveRuntime;
use dokime::operations::content as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::Error;

// A Quiet Follow Sends An Empty Chunk Well Inside The Client's 30 s Idle Deadline.
const FOLLOW_HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(10);

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::FollowExecutionFile),
}

impl Serve<wire::FollowExecutionFile> for Resources {
    fn serve<'a>(&'a self, request: wire::FollowExecutionFile, ctx: Context<'a, wire::FollowExecutionFile>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let runtime = &dokime::app::live_runtime(&self.runtime).await?;
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
                let cap = super::download_chunk_bytes(runtime, self.maximum)?;
                let template = dokime::domain::model::content::wire::ExecutionFileFollowFrame {
                    payload: Some(dokime::domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(
                        dokime::domain::model::content::wire::ArtifactChunk { offset: u64::MAX, data: Vec::new() },
                    )),
                };
                let transport_maximum = super::output_chunk_bytes(&template, cap, self.maximum)?;
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
                send(ctx.streams(), metadata, self.maximum).await?;
                loop {
                    let next = {
                        let mut pending = std::pin::pin!(follower.next_frame(maximum));
                        loop {
                            match tokio::time::timeout(FOLLOW_HEARTBEAT, &mut pending).await {
                                Ok(result) => break result.map_err(failure)?,
                                Err(_) => {
                                    let keepalive = dokime::domain::model::content::wire::ExecutionFileFollowFrame {
                                        payload: Some(dokime::domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(
                                            dokime::domain::model::content::wire::ArtifactChunk { offset: 0, data: Vec::new() },
                                        )),
                                    };
                                    send(ctx.streams(), Response::from(keepalive), self.maximum).await?;
                                }
                            }
                        }
                    };
                    let Some(frame) = next else { break };
                    let terminal =
                        matches!(&frame.payload, Some(dokime::domain::model::content::wire::ExecutionFileFollowFramePayload::Terminal(_)));
                    send(ctx.streams(), frame, self.maximum).await?;
                    if terminal {
                        tracing::debug!(execution_id = %execution,
                            elapsed_ms = started.elapsed().as_millis() as u64, "execution file follow terminal sent");
                    }
                }
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

    setup.resource::<Host<wire::FollowExecutionFile>>(Host::<wire::FollowExecutionFile>::new(served));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

async fn send(
    output: &StreamOutput<dokime::domain::model::content::wire::ExecutionFileFollowFrame>,
    frame: Response<dokime::domain::model::content::wire::ExecutionFileFollowFrame>, maximum: usize,
) -> Result<(), Status> {
    if super::output_frame_len(&*frame)? > maximum {
        return Err(Status::resource_exhausted("follow frame exceeds listener message limit"));
    }
    let sent = output.send_owned(frame).await;
    sent.map_err(crate::grpc::chunked::output_failure)
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

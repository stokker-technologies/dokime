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
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::error::detailed;
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::wire_checkpoint as checkpoint;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::execution_history::ExecutionCheckpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<history::GetExecutionCheckpoint> for Resources {
    fn serve<'a>(
        &'a self, request: history::GetExecutionCheckpoint, _: Context<'a, history::GetExecutionCheckpoint>,
    ) -> Served<'a, checkpoint::ExecutionCheckpoint> {
        Box::pin(async move {
            let id = TestExecutionId::from_uuid(request.execution_identifier.clone())
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            let response = self.runtime.execution_saved_checkpoint(id).await.map_err(|source| match source {
                dokime::runtime::Error::Admission(AdmissionFailure::NotFound { .. }) => {
                    detailed(Status::not_found("execution has no saved seal"), &source)
                }
                source => wire::query_failure(source),
            })?;
            project(&response, self.max_message_bytes).map(Into::into).map_err(|source| wire::query_failure(source.into()))
        })
    }
}

impl Serve<history::DeleteExecution> for Resources {
    fn serve<'a>(
        &'a self, mut request: history::DeleteExecution, _: Context<'a, history::DeleteExecution>,
    ) -> Served<'a, dokime::domain::model::history::wire::DeleteExecutionResponse> {
        Box::pin(async move {
            if self.runtime.watch_state().latest().state().shutting_down {
                return Err(Status::unavailable("listener is stopping"));
            }
            wire::checkpoint(request.checkpoint.take())?;
            wire::deletion_request_identifier(request.request_identifier.clone())?;
            Err(Status::unimplemented("execution deletion is not supported by native storage"))
        })
    }
}

/* === Public Functions === */

pub(in crate::grpc) async fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), crate::grpc::Error> {
    super::browse::register(runtime, max_message_bytes).await?;
    super::cycles::register(runtime, max_message_bytes).await?;
    super::read::register(runtime, max_message_bytes).await?;
    super::window::register(runtime, max_message_bytes).await?;
    super::telemetry::register(runtime, max_message_bytes).await?;
    super::records::register(runtime, max_message_bytes).await?;
    super::evidence::register(runtime, max_message_bytes).await?;
    super::capture::register(runtime, max_message_bytes).await?;
    super::context::register(runtime, max_message_bytes).await?;
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<history::GetExecutionCheckpoint>::new(served.clone()))?;
    dependencies.insert(Host::<history::DeleteExecution>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<history::GetExecutionCheckpoint>()
        .operation::<history::DeleteExecution>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(super) fn project(source: &ExecutionCheckpoint, maximum: usize) -> dokime::harmos::Result<checkpoint::ExecutionCheckpoint> {
    let value = wire::checkpoint_wire_ref(source, maximum).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        dokime::harmos::Error::backend("project inspected execution checkpoint", dokime::harmos::BackendFailure::Corrupt, source)
    })?;
    if value.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(value)
}

/* === Private Functions === */

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "historical checkpoint response",
    }
}

/* === Tests === */

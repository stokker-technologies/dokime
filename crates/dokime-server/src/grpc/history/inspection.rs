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
use dokime::harmos::Context;
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::error::detailed;
use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
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
    runtime: LiveRuntime,
    max_message_bytes: usize,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(history::GetExecutionCheckpoint),
    Operation1(history::DeleteExecution),
}

/* === Traits === */

/* === Implementations === */

impl Serve<history::GetExecutionCheckpoint> for Resources {
    fn serve<'a>(
        &'a self, request: history::GetExecutionCheckpoint, _: Context<'a, history::GetExecutionCheckpoint>,
    ) -> Served<'a, checkpoint::ExecutionCheckpoint> {
        Box::pin(async move {
            let id = TestExecutionId::from_uuid(request.execution_identifier.clone())
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            let response =
                dokime::app::live_runtime(&self.runtime).await?.execution_saved_checkpoint(id).await.map_err(|source| match source {
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
            if dokime::app::live_runtime(&self.runtime).await?.watch_state().latest().state().shutting_down {
                return Err(Status::unavailable("listener is stopping"));
            }
            wire::checkpoint(request.checkpoint.take())?;
            wire::deletion_request_identifier(request.request_identifier.clone())?;
            Err(Status::unimplemented("execution deletion is not supported by native storage"))
        })
    }
}

/* === Public Functions === */

pub(in crate::grpc) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    super::browse::prepare(setup, runtime, max_message_bytes)?;
    super::cycles::prepare(setup, runtime, max_message_bytes)?;
    super::read::prepare(setup, runtime, max_message_bytes)?;
    super::window::prepare(setup, runtime, max_message_bytes)?;
    super::telemetry::prepare(setup, runtime, max_message_bytes)?;
    super::records::prepare(setup, runtime, max_message_bytes)?;
    super::evidence::prepare(setup, runtime, max_message_bytes)?;
    super::capture::prepare(setup, runtime, max_message_bytes)?;
    super::context::prepare(setup, runtime, max_message_bytes)?;
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });

    setup.resource::<Host<history::GetExecutionCheckpoint>>(Host::<history::GetExecutionCheckpoint>::new(served.clone()));
    setup.resource::<Host<history::DeleteExecution>>(Host::<history::DeleteExecution>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
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

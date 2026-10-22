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
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::transcode::Transcode;
use crate::grpc::{error::detailed, value};
use dokime::RuntimeHandle;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::wire_checkpoint as checkpoint;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::operations::history;
use dokime::runtime::query::execution_history::ExecutionCheckpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    max_message_bytes: usize,
}

struct Inspect {
    resources: Arc<Resources>,
}

struct Delete {
    resources: Arc<Resources>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum Operations {
    Inspect(Inspect),
    Delete(Delete),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Inspect {
    type Declaration = history::GetExecutionCheckpoint;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut history::GetExecutionCheckpoint, _: &mut (),
    ) -> Result<Response<checkpoint::ExecutionCheckpoint>, Status> {
        value::text(&request.execution_identifier)?;
        let id = TestExecutionId::from_uuid(request.execution_identifier.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        let response = self.resources.runtime.execution_saved_checkpoint(id).await.map_err(|source| match source {
            dokime::runtime::Error::Admission(AdmissionFailure::NotFound { .. }) => {
                detailed(Status::not_found("execution has no saved seal"), &source)
            }
            source => wire::query_failure(source),
        })?;
        project(&response, self.resources.max_message_bytes).map(Into::into).map_err(|source| wire::query_failure(source.into()))
    }
}

impl Operation for Delete {
    type Declaration = history::DeleteExecution;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut history::DeleteExecution, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::DeleteExecutionResponse>, Status> {
        if self.resources.runtime.watch_state().latest().state().shutting_down {
            return Err(Status::unavailable("listener is stopping"));
        }
        wire::checkpoint(request.checkpoint.take())?;
        wire::deletion_request_identifier(request.request_identifier.clone())?;
        Err(Status::unimplemented("execution deletion is not supported by native storage"))
    }
}

/* === Public Functions === */

pub(in crate::grpc) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), crate::grpc::Error> {
    super::browse::register(runtime, max_message_bytes)?;
    super::cycles::register(runtime, max_message_bytes)?;
    super::read::register(runtime, max_message_bytes)?;
    super::window::register(runtime, max_message_bytes)?;
    super::telemetry::register(runtime, max_message_bytes)?;
    super::records::register(runtime, max_message_bytes)?;
    super::evidence::register(runtime, max_message_bytes)?;
    super::capture::register(runtime, max_message_bytes)?;
    super::context::register(runtime, max_message_bytes)?;
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), max_message_bytes }),
    )?;
    Ok(())
}

pub(super) fn project(source: &ExecutionCheckpoint, maximum: usize) -> dokime::harmos::Result<checkpoint::ExecutionCheckpoint> {
    let value = wire::checkpoint_wire_ref(source).map_err(|source| {
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
    dokime::harmos::Error::ResourceExhausted { resource: "historical checkpoint response" }
}

/* === Tests === */

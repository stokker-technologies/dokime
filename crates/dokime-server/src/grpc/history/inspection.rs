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
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos::Protobuf;
use harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::{error::detailed, value};
use dokime_app::RuntimeHandle;
use dokime_app::core::TestExecutionId;
use dokime_app::runtime::query::execution_history::ExecutionCheckpoint;
use dokime_contract::operations::{checkpoint, history};

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
#[harmos::catalog]
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
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut history::GetExecutionCheckpointRequest,
        _: &mut (),
    ) -> Result<Response<checkpoint::ExecutionCheckpoint>, Status> {
        value::text(&request.execution_identifier)?;
        let id = TestExecutionId::from_uuid(request.execution_identifier.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        let response = self
            .resources
            .runtime
            .execution_saved_checkpoint(id, |checkpoint, allowance| {
                project(checkpoint, allowance, self.resources.max_message_bytes)
            })
            .await
            .map_err(|source| match source {
                dokime_app::Error::NotFound { .. } => {
                    detailed(Status::not_found("execution has no saved seal"), &source)
                }
                source => wire::query_failure(source),
            })?;
        response
            .try_into_response()
            .map_err(|source| wire::query_failure(source.into()))
    }
}

impl Operation for Delete {
    type Declaration = history::DeleteExecution;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut history::DeleteExecutionRequest,
        _: &mut (),
    ) -> Result<Response<history::DeleteExecutionResponse>, Status> {
        if self
            .resources
            .runtime
            .watch_state()
            .latest()
            .state()
            .shutting_down
        {
            return Err(Status::unavailable("listener is stopping"));
        }
        wire::checkpoint(request.checkpoint.take())?;
        wire::deletion_request_identifier(request.request_identifier.clone())?;
        Err(Status::unimplemented(
            "execution deletion is not supported by native storage",
        ))
    }
}

/* === Public Functions === */
pub(in crate::grpc) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    super::browse::register(runtime, max_message_bytes)?;
    super::read::register(runtime, max_message_bytes)?;
    super::window::register(runtime, max_message_bytes)?;
    super::telemetry::register(runtime, max_message_bytes)?;
    super::records::register(runtime, max_message_bytes)?;
    super::evidence::register(runtime, max_message_bytes)?;
    super::capture::register(runtime, max_message_bytes)?;
    super::context::register(runtime, max_message_bytes)?;
    runtime
        .operations()
        .register_wire_operations::<Operations, Resources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(Resources {
                runtime: runtime.clone(),
                max_message_bytes,
            }),
        )?;
    Ok(())
}

/* === Private Functions === */
pub(super) fn project(
    source: &ExecutionCheckpoint,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<checkpoint::ExecutionCheckpoint> {
    let graph = [
        source.execution_id.as_str().len(),
        harmos::Message::encoded_len(&source.version),
        source.position.history.store.as_str().len(),
        source.position.history.epoch.as_str().len(),
        source.evidence_through.history.store.as_str().len(),
        source.evidence_through.history.epoch.as_str().len(),
    ]
    .into_iter()
    .try_fold(
        std::mem::size_of::<checkpoint::ExecutionCheckpoint>(),
        |bytes, field| bytes.checked_add(field).ok_or_else(exhausted),
    )?;
    if graph > allowance.get() {
        return Err(exhausted());
    }
    let value = wire::checkpoint_wire_ref(source).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "project inspected execution checkpoint",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    if Protobuf::encoded_len(&value) > maximum {
        return Err(exhausted());
    }
    Ok(value)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "historical checkpoint response",
    }
}

/* === Tests === */

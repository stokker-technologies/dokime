/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod wire;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{borrow::Borrow, num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamOutput},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
use tonic::Status;
// Local Imports
use super::value;
use dokime_app::{
    RuntimeHandle,
    core::{DocumentId, Identifier, TestCycleId, TestExecutionId},
};
use dokime_contract::operations::documents as contract;

/* === Constants === */
const CAPTURE_BYTES: usize = 32 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    admission: Arc<Semaphore>,
}
struct Capture {
    resources: Arc<Resources>,
}
struct UpdatePlotSettings {
    resources: Arc<Resources>,
}
struct FrameBuffer {
    frames: Vec<contract::DocumentFrame>,
    _permit: OwnedSemaphorePermit,
}
struct FrameView {
    buffer: Response<FrameBuffer>,
    index: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for FrameBuffer {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.frames
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.frames)),
            )
            .saturating_add(
                self.frames
                    .len()
                    .saturating_mul(4 * std::mem::size_of::<FrameView>()),
            )
    }
}
impl Borrow<contract::DocumentFrame> for FrameView {
    fn borrow(&self) -> &contract::DocumentFrame {
        &self.buffer.frames[self.index]
    }
}
impl Operation for Capture {
    type Declaration = contract::CaptureCurrentDocument;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<contract::DocumentFrame>;
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::CaptureCurrentDocumentRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let permit = self
            .resources
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::resource_exhausted("document resolver is busy"))?;
        let (cycle, execution) = owner(request.scope.take())?;
        value::text(&request.document_id)?;
        let id = DocumentId::from_identifier(&request.document_id)
            .map_err(|_| Status::invalid_argument("invalid document ID"))?;
        // A scope without an execution names the cycle's own document.
        let runtime = &self.resources.runtime;
        let prepared = match &execution {
            Some(execution) => runtime.prepare_current_document(&cycle, execution, &id),
            None => runtime.prepare_current_cycle_document(&cycle, &id),
        }
        .map_err(super::execution::status)?;
        let cancel = CancellationToken::new();
        let _cancel_on_drop = cancel.clone().drop_guard();
        let handle = tokio::runtime::Handle::current();
        let maximum = self.resources.maximum;
        let runtime = self.resources.runtime.clone();
        let buffer = tokio::task::spawn_blocking(move || {
            let captured = handle.block_on(async {
                tokio::select! { biased;
                    _ = cancel.cancelled() => Err(Status::cancelled("document request cancelled")),
                    result = prepared.capture() => result.map_err(super::execution::status),
                }
            })?;
            if cancel.is_cancelled() {
                return Err(Status::cancelled("document request cancelled"));
            }
            let preparation = runtime
                .storage()
                .prepare_production(NonZeroUsize::new(CAPTURE_BYTES).unwrap())
                .map_err(failure)?;
            let frames = wire::frames(captured, maximum)?;
            let buffer = FrameBuffer {
                frames: frames.into_iter().collect(),
                _permit: permit,
            };
            preparation
                .finish(buffer)
                .map_err(failure)?
                .try_into_value()
                .map_err(failure)?
                .try_into_response()
                .map_err(failure)
        })
        .await
        .map_err(|_| Status::internal("document worker failed"))??;
        for index in 0..buffer.frames.len() {
            // Each Projection Retains The Original Capture Loan And Native Resolver Permit.
            let frame = Response::from_owner(Arc::new(FrameView {
                buffer: buffer.clone(),
                index,
            }));
            output.send_owned(frame).await.map_err(failure)?;
        }
        Ok(().into())
    }
}

impl Operation for UpdatePlotSettings {
    type Declaration = contract::UpdatePlotSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::UpdatePlotSettingsRequest,
        _: &mut (),
    ) -> Result<Response<()>, Status> {
        let (cycle, execution) = scope(request.scope.take())?;
        value::text(&request.document_id)?;
        value::text(&request.plot_id)?;
        let document = DocumentId::from_identifier(&request.document_id)
            .map_err(|_| Status::invalid_argument("invalid document ID"))?;
        let plot = Identifier::new(&request.plot_id)
            .map_err(|_| Status::invalid_argument("invalid plot ID"))?;
        if request
            .display_points
            .is_some_and(|points| points > 100_000)
        {
            return Err(Status::invalid_argument(
                "display_points must not exceed 100000",
            ));
        }
        self.resources
            .runtime
            .update_plot_settings(
                &cycle,
                &execution,
                &document,
                &plot,
                request.display_points,
                request.downsample,
            )
            .await
            .map_err(super::execution::status)?;
        Ok(().into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
        admission: Arc::new(Semaphore::new(1)),
    });
    runtime.operations().register_wire_operation::<Capture>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime
        .operations()
        .register_wire_operation::<UpdatePlotSettings>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources,
        )?;
    Ok(())
}

/* === Private Functions === */
/// The capture scope: a cycle, and the execution whose document it names, or
/// none for the cycle's own document.
fn owner(
    scope: Option<contract::DocumentScope>,
) -> Result<(TestCycleId, Option<TestExecutionId>), Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("document scope required"))?;
    let cycle = TestCycleId::from_uuid(scope.cycle_id)
        .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    if scope.execution_id.is_empty() {
        return Ok((cycle, None));
    }
    let execution = TestExecutionId::from_uuid(scope.execution_id)
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    Ok((cycle, Some(execution)))
}

fn scope(scope: Option<contract::DocumentScope>) -> Result<(TestCycleId, TestExecutionId), Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("document scope required"))?;
    let cycle = TestCycleId::from_uuid(scope.cycle_id)
        .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    let execution = TestExecutionId::from_uuid(scope.execution_id)
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    Ok((cycle, execution))
}
fn failure(error: harmos::Error) -> Status {
    let status = if matches!(error, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("document admission failed")
    } else {
        Status::internal("document admission failed")
    };
    super::error::detailed(status, &error.into())
}

/* === Tests === */

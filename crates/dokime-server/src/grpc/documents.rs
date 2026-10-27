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
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response, StreamOutput};
use tokio_util::sync::CancellationToken;
use tonic::Status;

// Local Imports
use dokime::RuntimeHandle;
use dokime::domain::model::identity::DocumentId;
use dokime::domain::model::identity::Identifier;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::operations::documents as contract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Capture {
    resources: Arc<Resources>,
}

struct UpdatePlotSettings {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Capture {
    type Declaration = contract::CaptureCurrentDocument;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::documents::wire::DocumentFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, request: &mut contract::CaptureCurrentDocument, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            let (cycle, execution) = owner(request.scope.take())?;
            let id = DocumentId::from_identifier(&request.document_id).map_err(|_| Status::invalid_argument("invalid document ID"))?;
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
            let frames = tokio::task::spawn_blocking(move || {
                let captured = handle.block_on(async {
                    tokio::select! { biased;
                        _ = cancel.cancelled() => Err(Status::cancelled("document request cancelled")),
                        result = prepared.capture() => result.map_err(super::execution::status),
                    }
                })?;
                if cancel.is_cancelled() {
                    return Err(Status::cancelled("document request cancelled"));
                }
                wire::frames(captured, maximum)
            })
            .await
            .map_err(|_| Status::internal("document worker failed"))??;
            for frame in frames {
                output.send_owned(frame.into()).await.map_err(failure)?;
            }
            Ok(().into())
        })
        .await
    }
}

impl Operation for UpdatePlotSettings {
    type Declaration = contract::UpdatePlotSettings;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::UpdatePlotSettings, _: &mut (),
    ) -> Result<Response<()>, Status> {
        let (cycle, execution) = scope(request.scope.take())?;
        let document = DocumentId::from_identifier(&request.document_id).map_err(|_| Status::invalid_argument("invalid document ID"))?;
        let plot = Identifier::new(&request.plot_id).map_err(|_| Status::invalid_argument("invalid plot ID"))?;
        self.resources
            .runtime
            .update_plot_settings(&cycle, &execution, &document, &plot, request.display_points, request.downsample)
            .await
            .map_err(super::execution::status)?;
        Ok(().into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum });
    runtime.operations().register_wire_operation::<Capture>("dokime", dokime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<UpdatePlotSettings>("dokime", dokime::app::Lanes::Open, resources)?;
    Ok(())
}

/* === Private Functions === */

fn owner(scope: Option<dokime::domain::model::documents::wire::DocumentScope>) -> Result<(TestCycleId, Option<TestExecutionId>), Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("document scope required"))?;
    let cycle = TestCycleId::from_uuid(scope.cycle_id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    if scope.execution_id.is_empty() {
        return Ok((cycle, None));
    }
    let execution = TestExecutionId::from_uuid(scope.execution_id).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    Ok((cycle, Some(execution)))
}

fn scope(scope: Option<dokime::domain::model::documents::wire::DocumentScope>) -> Result<(TestCycleId, TestExecutionId), Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("document scope required"))?;
    let cycle = TestCycleId::from_uuid(scope.cycle_id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    let execution = TestExecutionId::from_uuid(scope.execution_id).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    Ok((cycle, execution))
}

fn failure(error: dokime::harmos::Error) -> Status {
    let status = if matches!(error, dokime::harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("document admission failed")
    } else {
        Status::internal("document admission failed")
    };
    super::error::detailed(status, &error.into())
}

/* === Tests === */

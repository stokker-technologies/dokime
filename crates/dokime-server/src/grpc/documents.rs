/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod wire;

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt};
use tokio_util::sync::CancellationToken;
use tonic::Status;

use dokime::RuntimeHandle;
use dokime::domain::model::identity::DocumentId;
use dokime::domain::model::identity::Identifier;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::documents as contract;
use dokime::operations::hosting::{Host, Serve, Served};

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

impl Serve<contract::CaptureCurrentDocument> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::CaptureCurrentDocument, ctx: Context<'a, contract::CaptureCurrentDocument>,
    ) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let (cycle, execution) = owner(request.scope.take())?;
                let id = DocumentId::from_identifier(&request.document_id).map_err(|_| Status::invalid_argument("invalid document ID"))?;
                // A scope without an execution names the cycle's own document.
                let runtime = &self.runtime;
                let prepared = match &execution {
                    Some(execution) => runtime.prepare_current_document(&cycle, execution, &id),
                    None => runtime.prepare_current_cycle_document(&cycle, &id),
                }
                .map_err(super::execution::status)?;
                let cancel = CancellationToken::new();
                let _cancel_on_drop = cancel.clone().drop_guard();
                let handle = tokio::runtime::Handle::current();
                let maximum = self.maximum;
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
                    ctx.streams().send_owned(frame.into()).await.map_err(crate::grpc::chunked::output_failure)?;
                }
                Ok(().into())
            })
            .await
        })
    }
}

impl Serve<contract::UpdatePlotSettings> for Resources {
    fn serve<'a>(&'a self, mut request: contract::UpdatePlotSettings, _: Context<'a, contract::UpdatePlotSettings>) -> Served<'a, ()> {
        Box::pin(async move {
            let (cycle, execution) = scope(request.scope.take())?;
            let document =
                DocumentId::from_identifier(&request.document_id).map_err(|_| Status::invalid_argument("invalid document ID"))?;
            let plot = Identifier::new(&request.plot_id).map_err(|_| Status::invalid_argument("invalid plot ID"))?;
            self.runtime
                .update_plot_settings(&cycle, &execution, &document, &plot, request.display_points, request.downsample)
                .await
                .map_err(super::execution::status)?;
            Ok(().into())
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<contract::CaptureCurrentDocument>::new(served.clone()))?;
    dependencies.insert(Host::<contract::UpdatePlotSettings>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<contract::CaptureCurrentDocument>()
        .operation::<contract::UpdatePlotSettings>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

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

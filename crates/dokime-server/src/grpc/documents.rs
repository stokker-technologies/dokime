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
use dokime::harmos::Context;
use tokio_util::sync::CancellationToken;
use tonic::Status;

// Local Imports
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::DocumentId;
use dokime::domain::model::identity::Identifier;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::documents as contract;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

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
                let runtime = &dokime::app::live_runtime(&self.runtime).await?;
                let prepared = match &execution {
                    Some(execution) => runtime.prepare_current_document(&cycle, execution, &id),
                    None => runtime.prepare_current_cycle_document(&cycle, &id),
                }
                .map_err(super::execution::status)?;
                let query = dokime::CurrentDocumentQuery {
                    structure_only: request.structure_only,
                    cards: request
                        .card_ids
                        .iter()
                        .map(|id| Identifier::new(id).map_err(|_| Status::invalid_argument("invalid card ID")))
                        .collect::<Result<_, _>>()?,
                };
                let cancel = CancellationToken::new();
                let _cancel_on_drop = cancel.clone().drop_guard();
                let handle = tokio::runtime::Handle::current();
                let maximum = self.maximum;
                let (sender, mut frames) = tokio::sync::mpsc::channel(1);
                let worker = tokio::task::spawn_blocking(move || {
                    handle.block_on(async move {
                        let (events, mut received) = tokio::sync::mpsc::channel(1);
                        let mut capture = Some(Box::pin(prepared.query_stream(query, events)));
                        let mut stream = wire::Stream::new(maximum);
                        let mut closed = false;
                        while capture.is_some() || !closed {
                            tokio::select! { biased;
                                _ = cancel.cancelled() => return Err(Status::cancelled("document request cancelled")),
                                event = received.recv(), if !closed => {
                                    match event {
                                        Some(event) => for frame in stream.event(event)? {
                                            sender.send(frame).await.map_err(|_| Status::cancelled("document receiver closed"))?;
                                        },
                                        None => closed = true,
                                    }
                                },
                                result = async { capture.as_mut().expect("capture is running").await }, if capture.is_some() => {
                                    result.map_err(super::execution::status)?;
                                    capture = None;
                                },
                            }
                        }
                        sender.send(stream.completion()).await.map_err(|_| Status::cancelled("document receiver closed"))?;
                        Ok::<_, Status>(())
                    })
                });
                while let Some(frame) = frames.recv().await {
                    ctx.streams().send_owned(frame.into()).await.map_err(crate::grpc::chunked::output_failure)?;
                }
                worker.await.map_err(|_| Status::internal("document worker failed"))??;
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
            dokime::app::live_runtime(&self.runtime)
                .await?
                .update_plot_settings(&cycle, &execution, &document, &plot, request.display_points, request.downsample)
                .await
                .map_err(super::execution::status)?;
            Ok(().into())
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), super::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<contract::CaptureCurrentDocument>>(Host::<contract::CaptureCurrentDocument>::new(served.clone()));
    setup.resource::<Host<contract::UpdatePlotSettings>>(Host::<contract::UpdatePlotSettings>::new(served));

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

/* === Tests === */

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod cycle;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use futures_util::future::BoxFuture;
use harmos::Context;
use tonic::Status;

// Local Imports
use crate::operations::hosting::{Host, Serve, Served};
use crate::operations::reports::MaterializeReport;
use crate::report::Scope;
use crate::{RuntimeHandle, app::Lanes, runtime::Error, runtime::Result, runtime::storage::executions::ExecutionStorage};
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::reports::wire::MaterializedReport;
use dokime_domain::model::reports::wire::ReportSubject;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Resources {
    runtime: RuntimeHandle,
    pub(super) execution_storage: Arc<ExecutionStorage>,
    pub(super) settings: crate::storage::sqlite::State,
    pub(super) providers: Arc<crate::domains::execution::provider::ProviderExecutorRegistry>,
    pub(super) progress: Arc<crate::runtime::progress::OperationRegistry>,
    scope: Scope,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Materialize(MaterializeReport),
}

/* === Traits === */

/* === Implementations === */

struct Runtime(crate::app::LiveRuntime);

impl Serve<MaterializeReport> for Runtime {
    fn serve<'a>(&'a self, request: MaterializeReport, _: Context<'a, MaterializeReport>) -> Served<'a, MaterializedReport> {
        Box::pin(async move {
            let runtime = crate::app::live_runtime(&self.0).await?;
            let resources = Resources::from_runtime(&runtime);
            resources
                .materialize(request)
                .await
                .inspect_err(|error| {
                    tracing::error!(?error, "report materialization failed");
                })
                .map(Into::into)
                .map_err(operation_status)
        })
    }
}

impl Resources {
    // Freeze Catalog Contributions And Plugin Analysis Results Before Completion Releases Plugins.
    pub(crate) fn freeze_report_context<'a>(
        &'a self, source: &'a dokime_domain::model::execution::result::CycleView,
        inputs: &'a crate::runtime::storage::executions::CaptureInputs,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let id = &inputs.execution.test_execution_id;
            let mut lifecycle = self
                .execution_storage
                .report_finalization(id)
                .await?
                .and_then(|intent| intent.lifecycle)
                .ok_or_else(|| invalid("report context requires a frozen lifecycle"))?;
            if lifecycle.context.is_some() {
                return Ok(());
            }
            let defaults = self.settings.read_application_settings().await?.settings.report;
            let inspection = self.execution_storage.capture_execution_view(source, inputs, defaults).await?;
            let mut captured = inspection.source_snapshot()?.clone();
            let progress = self.progress.begin("Freezing Report Analyses");
            captured.frozen_plots = crate::report::materialize::freeze_analyses(&self.scope, &inspection, &progress).await?;
            progress.finish();
            let context = crate::report::saved::save_context(&self.execution_storage.storage, &captured).await?;
            lifecycle.context = Some(context);
            self.runtime
                .commit(crate::transactions::CaptureReportLifecycle { lifecycle })
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
            self.execution_storage.storage.flush_recovery().await?;
            Ok(())
        })
    }

    // Keep Nested Report Frames Off Native Control And Listener Stacks.
    pub(crate) fn materialize(&self, request: MaterializeReport) -> BoxFuture<'_, Result<MaterializedReport>> {
        Box::pin(async move {
            let scope = &self.scope;
            if !self.execution_storage.storage.cursor_saved().covers(&request.at)? {
                return Err(invalid("requested report cursor is not durable"));
            }
            // Materializing A Report Is One Operation; Its Callers Add None Around It.
            match &request.subject {
                ReportSubject::Execution(id) => {
                    let id = TestExecutionId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                    let progress = self.progress.begin("Materializing Execution Report");
                    progress.track(execution(scope, self, &id, &request.at, &progress).await.map_err(report_failure))
                }
                ReportSubject::Cycle(id) => {
                    let id = TestCycleId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                    let progress = self.progress.begin("Materializing Cycle Report");
                    progress.track(cycle::materialize(scope, self, &id, &request.at, &progress).await.map_err(report_failure))
                }
            }
        })
    }
}

impl RuntimeHandle {
    /// Render a selected document at an already captured evidence frontier.
    /// The returned temporary HTML neither publishes a report nor seals evidence.
    pub async fn render_execution_document(
        &self, snapshot: &crate::runtime::ExecutionView, document: crate::document::DocumentDefinition,
    ) -> Result<crate::report::RenderedExecutionReport> {
        if !self.storage().cursor_saved().covers(snapshot.evidence_through()?)? {
            return Err(invalid("document snapshot exceeds this backend's saved evidence"));
        }
        let inspection = snapshot.with_document(document)?;
        let resources = Resources::from_runtime(self);
        let progress = resources.progress.begin("Rendering Document");
        let captured = progress.track(crate::report::materialize::document(&resources.scope, &inspection, &progress).await)?;
        let outputs = inspection.outputs()?.to_vec();
        let saved = inspection.saved().clone();
        let rendering = tokio::task::spawn_blocking(move || crate::report::Renderer.execution(captured, outputs, &saved)).await;
        let rendered = progress.track(rendering)?;
        let rendered = progress.track(rendered)?;
        progress.finish();
        Ok(rendered)
    }
}

/* === Public Functions === */

pub(crate) fn prepare(setup: &mut harmos::Setup<crate::app::Dokime>, live: &crate::app::LiveRuntime) {
    setup.resource::<Host<MaterializeReport>>(Host::<MaterializeReport>::new(Arc::new(Runtime(live.clone()))));
    setup.operations::<Operations>(Lanes::Open);
}

impl Resources {
    pub(crate) fn from_runtime(runtime: &RuntimeHandle) -> Self {
        Self {
            runtime: runtime.clone(),
            execution_storage: runtime.execution_storage.clone(),
            settings: runtime.local_storage.clone(),
            providers: runtime.providers.clone(),
            progress: runtime.operation_progress().clone(),
            scope: Scope::new(
                runtime.state_manager.clone(),
                crate::app::Resources {
                    schema_settings: *runtime.schema_settings(),
                    analysis: crate::report::analysis::Resources::from_runtime(runtime),
                    renderer: crate::report::Renderer,
                },
                runtime.local_storage.clone(),
                runtime.query_config().clone(),
            ),
        }
    }
}

#[cfg(feature = "testing")]
impl RuntimeHandle {
    // Exercise The Same Historical Materializer Used By Cycle Composition.
    pub async fn rebuild_sealed_report(&self, id: &TestExecutionId) -> Result<harmos::storage::FileReference> {
        let resources = Resources::from_runtime(self);
        let view = crate::runtime::storage::execution::ExecutionView::open(self.storage(), id, harmos::storage::ObjectVersion::Latest)
            .await?
            .ok_or_else(|| Error::InvalidReportInputs("sealed execution is missing".into()))?;
        let seal = view.execution().saved.as_ref().ok_or_else(|| Error::InvalidReportInputs("execution is unsealed".into()))?;
        let progress = resources.progress.begin("Rebuilding Sealed Report");
        let captured = crate::report::materialize::sealed_member(&resources.scope, self.storage(), seal, &progress).await?;
        let rendered = crate::report::Renderer.execution(captured, seal.source.outputs.clone(), &self.storage().cursor_saved())?;
        let mut prepared = rendered.prepare_publication();
        let rebuilt = prepared.save_dependencies(self.storage(), &seal.sealed_at, &seal.source.execution).await?;
        Ok(rebuilt.report)
    }
}

/* === Private Functions === */

fn report_failure(error: Error) -> Error {
    match error {
        Error::Admission(_) | Error::ShuttingDown | Error::Interrupted => error,
        _ => Error::ReportUnavailable(Box::new(error)),
    }
}

async fn execution(
    ctx: &Scope, resources: &Resources, id: &TestExecutionId, at: &harmos::storage::StorageCursor,
    progress: &crate::runtime::progress::Operation,
) -> Result<MaterializedReport> {
    let mut shutdown = resources.runtime.shutdown_requested.subscribe();
    if *shutdown.borrow() {
        return Err(Error::ShuttingDown);
    }
    progress.phase("Capturing Evidence");
    let app = ctx.resources();
    if resources.execution_storage.acquisition(id).await.is_none() {
        let source = resources.execution_storage.report_member_source(id).await?;
        let member = source
            .test_executions
            .iter()
            .find(|member| member.test_execution_id == *id)
            .ok_or_else(|| invalid("execution is not in its cycle"))?;
        resources.execution_storage.acquire(&source, member).await?;
    }
    let gate = resources.execution_storage.materialization_gate(id).await?;
    let _guard = gate.lock().await;
    // A Persisted Finalization Boundary Is the Report's Boundary, Also One Persisted After
    // The Caller Chose `at`; Its Lifecycle Is Read Before The Member So Both Agree.
    let intent = resources.execution_storage.report_finalization(id).await?;
    let at = intent.as_ref().map_or(at, |intent| &intent.evidence_at);
    // Recheck Mutable Lifecycle After Admission; Sealing Uses This Same Gate.
    let source = resources.execution_storage.report_member_source(id).await?;
    let member = source
        .test_executions
        .iter()
        .find(|member| member.test_execution_id == *id)
        .ok_or_else(|| invalid("execution disappeared during report admission"))?;
    // A Final Report Is Served As Published, Whether Or Not The Cycle State Holds Its Seal Yet.
    if let Some(seal) = &member.saved {
        return Ok(MaterializedReport { report: seal.report.clone() });
    }
    if let Some(seal) = resources.execution_storage.final_seal(id).await? {
        let report = seal.report.clone();
        resources.runtime.complete_report_publication(id).await?;
        return Ok(MaterializedReport { report });
    }
    if let Some(report) = resources.execution_storage.published_final_report(id, at).await? {
        resources.runtime.complete_report_publication(id).await?;
        return Ok(report);
    }
    let final_report = intent.as_ref().is_some_and(|intent| intent.lifecycle.is_some())
        && member.finished_at.is_some()
        && member.active_step_attempt.is_none()
        && !matches!(
            member.status,
            dokime_domain::model::execution::result::ExecutionStatus::Pending
                | dokime_domain::model::execution::result::ExecutionStatus::Running
        );
    if final_report {
        if let Some(reason) = resources.providers.final_report_refusal(member) {
            return Err(Error::InvalidReportInputs(reason));
        }
        resources.execution_storage.finish_outputs(id).await?;
    }
    let inputs = resources.execution_storage.prepare_capture(&source, member, at, final_report).await?;
    if final_report {
        resources.freeze_report_context(&source, &inputs).await?;
    }
    let defaults = resources.settings.read_application_settings().await?.settings.report;
    let inspection = resources.execution_storage.capture_execution_view(&source, &inputs, defaults).await?;

    if *inspection.evidence_through()? != inputs.through {
        return Err(invalid("capture changed requested report boundary"));
    }
    let storage = &resources.execution_storage.storage;
    let captured = tokio::select! {
        result = crate::report::materialize::execution(ctx, &inspection, final_report, inputs.current, progress) => result?,
        _ = shutdown.wait_for(|requested| *requested) => return Err(Error::ShuttingDown),
    };
    progress.phase("Rendering");
    let outputs = inputs.outputs.clone();
    let renderer = app.renderer;
    let saved = storage.cursor_saved();
    let rendering = tokio::task::spawn_blocking(move || renderer.execution_with_outputs(captured, outputs, &saved));
    let rendered = tokio::select! {
        result = rendering => result??,
        _ = shutdown.wait_for(|requested| *requested) => return Err(Error::ShuttingDown),
    };
    progress.phase("Publishing Report");
    let mut prepared = rendered.prepare_publication();
    if final_report {
        let report = resources.execution_storage.materialize_final_report(inspection, prepared, crate::model::time::now_utc()).await?;
        resources.runtime.complete_report_publication(id).await?;
        Ok(report)
    } else {
        let revision = resources.execution_storage.publication_revision(inspection.execution()).await?;
        prepared.publish_current(storage, &revision).await
    }
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

// Call Sites Own Diagnostics; Conversion Preserves the Transport Code and Message.
pub(crate) fn operation_status(error: Error) -> Status {
    let code = match &error {
        Error::Admission(failure) => {
            use dokime_domain::model::diagnostics::admission::AdmissionFailure as Failure;
            match failure {
                Failure::Validation { .. } => tonic::Code::InvalidArgument,
                Failure::Conflict { .. } => tonic::Code::FailedPrecondition,
                Failure::Overload { .. } => tonic::Code::ResourceExhausted,
                Failure::Unavailable { .. } => tonic::Code::Unavailable,
                Failure::Forbidden { .. } => tonic::Code::PermissionDenied,
                Failure::NotFound { .. } => tonic::Code::NotFound,
                Failure::Timeout { .. } => tonic::Code::DeadlineExceeded,
                Failure::Cancelled { .. } => tonic::Code::Cancelled,
                Failure::Internal { .. } => tonic::Code::Internal,
            }
        }
        Error::InvalidReportInputs(_) | Error::InvalidCheckpointQuery(_) => tonic::Code::InvalidArgument,
        Error::ShuttingDown | Error::Interrupted => tonic::Code::Unavailable,
        Error::ReportUnavailable(_) => tonic::Code::FailedPrecondition,
        _ => tonic::Code::FailedPrecondition,
    };
    let message = if matches!(&error, Error::ReportUnavailable(_) | Error::Admission(_) | Error::ShuttingDown | Error::Interrupted) {
        error.to_string()
    } else {
        "The report could not be created. Try again; if the problem continues, collect diagnostics.".into()
    };
    Status::new(code, message)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::diagnostics::admission::AdmissionFailure as Failure;

    #[test]
    fn operation_status_preserves_admission_codes_and_messages() {
        let failures = [
            (Failure::validation("invalid", "invalid input", vec![]), tonic::Code::InvalidArgument),
            (Failure::conflict("conflict", "conflicting input"), tonic::Code::FailedPrecondition),
            (Failure::overload("file_capture_cap", "capacity reached"), tonic::Code::ResourceExhausted),
            (Failure::unavailable("unavailable", "unavailable source"), tonic::Code::Unavailable),
            (Failure::forbidden("forbidden", "forbidden source"), tonic::Code::PermissionDenied),
            (Failure::not_found("file_capture_missing", "missing source"), tonic::Code::NotFound),
            (Failure::timeout("timeout", "deadline reached"), tonic::Code::DeadlineExceeded),
            (Failure::cancelled("cancelled", "cancelled request"), tonic::Code::Cancelled),
            (Failure::internal("internal", "internal failure"), tonic::Code::Internal),
        ];
        for (failure, code) in failures {
            let message = failure.to_string();
            let status = operation_status(Error::Admission(failure));
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), message);
        }
    }

    #[test]
    fn operation_status_preserves_runtime_codes_and_messages() {
        let fallback = "The report could not be created. Try again; if the problem continues, collect diagnostics.";
        let errors = [
            (Error::InvalidReportInputs("invalid".into()), tonic::Code::InvalidArgument, Some(fallback)),
            (Error::InvalidCheckpointQuery("invalid".into()), tonic::Code::InvalidArgument, Some(fallback)),
            (Error::ShuttingDown, tonic::Code::Unavailable, None),
            (Error::Interrupted, tonic::Code::Unavailable, None),
            (Error::ReportUnavailable(Box::new(Error::Interrupted)), tonic::Code::FailedPrecondition, None),
            (Error::Operation("failed".into()), tonic::Code::FailedPrecondition, Some(fallback)),
        ];
        for (error, code, message) in errors {
            let message = message.map(str::to_owned).unwrap_or_else(|| error.to_string());
            let status = operation_status(error);
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), message);
        }
    }
}

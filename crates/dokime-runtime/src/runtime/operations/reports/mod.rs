/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

mod cycle;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::{Context, RegistrationExt};
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

pub(super) struct Resources {
    pub(super) execution_storage: Arc<ExecutionStorage>,
    pub(super) settings: crate::storage::sqlite::State,
    pub(super) providers: Arc<crate::domains::execution::provider::ProviderExecutorRegistry>,
    pub(super) progress: Arc<crate::runtime::progress::OperationRegistry>,
    scope: Scope,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<MaterializeReport> for Resources {
    fn serve<'a>(&'a self, request: MaterializeReport, _: Context<'a, MaterializeReport>) -> Served<'a, MaterializedReport> {
        Box::pin(async move { self.materialize(request).await.map(Into::into).map_err(status) })
    }
}

impl Resources {
    async fn materialize(&self, request: MaterializeReport) -> Result<MaterializedReport> {
        let scope = &self.scope;
        if !self.execution_storage.storage.cursor_saved().covers(&request.at)? {
            return Err(invalid("requested report cursor is not durable"));
        }
        // Materializing A Report Is One Operation; Its Callers Add None Around It.
        match &request.subject {
            ReportSubject::Execution(id) => {
                let id = TestExecutionId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                let progress = self.progress.begin("Materializing Execution Report");
                progress.track(execution(scope, self, &id, &request.at, &progress).await)
            }
            ReportSubject::Cycle(id) => {
                let id = TestCycleId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                let progress = self.progress.begin("Materializing Cycle Report");
                progress.track(cycle::materialize(scope, self, &id, &request.at, &progress).await)
            }
        }
    }
}

/* === Public Functions === */

pub(crate) async fn register(runtime: &RuntimeHandle) -> Result<()> {
    let resources = Arc::new(Resources {
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
        ),
    });
    let mut dependencies = harmos::Dependencies::new();
    dependencies.insert(Host::<MaterializeReport>::new(resources))?;
    let batch = harmos::RegistrationBatch::new(dependencies).operation::<MaterializeReport>();
    runtime.operations.register(batch.lane(Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

async fn execution(
    ctx: &Scope, resources: &Resources, id: &TestExecutionId, at: &harmos::storage::StorageCursor,
    progress: &crate::runtime::progress::Operation,
) -> Result<MaterializedReport> {
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
        return Ok(MaterializedReport { report: seal.report.clone() });
    }
    if let Some(report) = resources.execution_storage.published_final_report(id, at).await? {
        return Ok(report);
    }
    let has_lifecycle = resources.execution_storage.has_report_lifecycle(id).await?;
    let final_report = has_lifecycle
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
    let defaults = resources.settings.read_application_settings().await?.settings.report;
    let inspection = resources.execution_storage.capture_execution_view(&source, &inputs, defaults).await?;

    if inspection.evidence_through()? != at {
        return Err(invalid("capture changed requested report boundary"));
    }
    let storage = &resources.execution_storage.storage;
    let captured = crate::report::materialize::execution(ctx, &inspection, final_report, progress).await?;
    progress.phase("Rendering");
    let outputs = inputs.outputs.clone();
    let renderer = app.renderer;
    let saved = storage.cursor_saved();
    let mut prepared = tokio::task::spawn_blocking(move || renderer.execution_with_outputs(captured, outputs, &saved)).await??;
    progress.phase("Publishing Report");
    if final_report {
        resources.execution_storage.materialize_final_report(ctx, inspection, prepared, crate::model::time::now_utc()).await
    } else {
        let revision = resources.execution_storage.publication_revision(inspection.execution()).await?;
        prepared.publish_current(storage, &revision, &ctx.resources().schema_settings).await
    }
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

// The Typed Error Stays Attached So Diagnostics Capture Still Reads It.
pub(crate) fn status(error: Error) -> Status {
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
        _ => tonic::Code::Internal,
    };
    let mut status = Status::new(code, error.to_string());
    status.set_source(Arc::new(error));
    status
}

/* === Tests === */

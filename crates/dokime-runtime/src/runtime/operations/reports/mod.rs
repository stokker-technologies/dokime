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
use harmos::operations::{Context, Operation};

// Local Imports
use crate::operations::reports::MaterializeReport;
use crate::{
    RuntimeHandle,
    app::{Dokime, Lanes},
    runtime::Error,
    runtime::Result,
    runtime::storage::executions::ExecutionStorage,
};
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::reports::wire::MaterializedReport;
use dokime_domain::model::reports::wire::ReportSubject;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Handler {
    resources: Arc<Resources>,
}

pub(super) struct Resources {
    pub(super) execution_storage: Arc<ExecutionStorage>,
    pub(super) settings: crate::config::Settings,
    pub(super) providers: Arc<crate::domains::execution::provider::ProviderExecutorRegistry>,
    pub(super) progress: Arc<crate::runtime::progress::OperationRegistry>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Handler {
    type Declaration = MaterializeReport;
    type Resources = Resources;
    type Error = Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, ctx: Self::Context<'a>, request: &mut MaterializeReport, _: &mut (),
    ) -> Result<harmos::operations::Response<MaterializedReport>> {
        let storage = ctx.scope().storage::<Dokime>().ok_or_else(|| invalid("report storage is unavailable"))?;
        if !storage.cursor_saved().covers(&request.at)? {
            return Err(invalid("requested report cursor is not durable"));
        }
        // Materializing A Report Is One Operation; Its Callers Add None Around It.
        let result = match &request.subject {
            ReportSubject::Execution(id) => {
                let id = TestExecutionId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                let progress = self.resources.progress.begin("Materializing Execution Report");
                progress.track(execution(&ctx, &self.resources, &id, &request.at, &progress).await)
            }
            ReportSubject::Cycle(id) => {
                let id = TestCycleId::from_uuid(id.clone()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
                let progress = self.resources.progress.begin("Materializing Cycle Report");
                progress.track(cycle::materialize(&ctx, &self.resources, &id, &request.at, &progress).await)
            }
        }?;
        Ok(result.into())
    }
}

/* === Public Functions === */

pub(crate) fn register(runtime: &RuntimeHandle) -> Result<()> {
    let resources = Resources {
        execution_storage: runtime.execution_storage.clone(),
        settings: runtime.settings().clone(),
        providers: runtime.providers.clone(),
        progress: runtime.operation_progress().clone(),
    };
    runtime.operations.register_wire_operation::<Handler>("dokime", Lanes::Open, Arc::new(resources))?;
    Ok(())
}

/* === Private Functions === */

async fn execution(
    ctx: &Context<'_>, resources: &Resources, id: &TestExecutionId, at: &harmos::storage::StorageCursor,
    progress: &crate::runtime::progress::Operation,
) -> Result<MaterializedReport> {
    progress.phase("Capturing Evidence");
    let app = ctx.scope().resources::<Dokime>();
    let source = resources.execution_storage.report_source(id).await?;
    let member = source
        .test_executions
        .iter()
        .find(|member| member.test_execution_id == *id)
        .ok_or_else(|| invalid("execution is not in its cycle"))?;
    if resources.execution_storage.acquisition(id).await.is_none() {
        resources.execution_storage.acquire(&source, member).await?;
    }
    let gate = resources.execution_storage.materialization_gate(id).await?;
    let _guard = gate.lock().await;
    // Recheck Mutable Lifecycle After Admission; Sealing Uses This Same Gate.
    let source = resources.execution_storage.report_source(id).await?;
    let member = source
        .test_executions
        .iter()
        .find(|member| member.test_execution_id == *id)
        .ok_or_else(|| invalid("execution disappeared during report admission"))?;
    // A Final Report Is Served As Published, Whether Or Not The Cycle State Holds Its Seal Yet.
    if let Some(seal) = &member.saved {
        return Ok(MaterializedReport { report: seal.report.clone() });
    }
    if let Some(seal) = resources.execution_storage.final_seal(id).await {
        return Ok(MaterializedReport { report: seal.report });
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
    let defaults = resources.settings.read(|settings| settings.report);
    let inspection = resources.execution_storage.capture_execution_view(&source, &inputs, defaults).await?;
    if inspection.evidence_through()? != at {
        return Err(invalid("capture changed requested report boundary"));
    }
    let storage = &resources.execution_storage.storage;
    let captured = Arc::new(crate::report::materialize::execution(ctx, &app.storage_settings, &inspection, final_report, progress).await?);
    progress.phase("Rendering");
    let outputs = inspection.outputs()?.to_vec();
    let renderer = app.renderer;
    let saved = storage.cursor_saved();
    let mut prepared = tokio::task::spawn_blocking(move || renderer.execution(captured, outputs, &saved)).await??;
    progress.phase("Publishing Report");
    if final_report {
        resources.execution_storage.materialize_final_report(ctx, inspection, prepared, crate::model::time::now_utc()).await
    } else {
        let revision = resources.execution_storage.publication_revision(inspection.execution()).await?;
        prepared.publish_current(storage, ctx, &revision).await
    }
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

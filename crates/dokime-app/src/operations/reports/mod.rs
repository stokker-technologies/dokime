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
use crate::{
    Error, Result, RuntimeHandle,
    app::{Dokime, Lanes},
    runtime::storage::executions::ExecutionStorage,
};
use dokime::core::{TestCycleId, TestExecutionId, Timestamp};
use dokime::operations::reports::{
    MaterializeReport, MaterializeReportRequest, MaterializedReport, ReportSubject,
};
use harmos::MemorySize;
use harmos::operations::{Context, Operation};
use std::sync::Arc;
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
        &mut self,
        ctx: Self::Context<'a>,
        request: &mut MaterializeReportRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<MaterializedReport>> {
        let storage = ctx
            .scope()
            .storage::<Dokime>()
            .ok_or_else(|| invalid("report storage is unavailable"))?;
        if !storage.cursor_saved().covers(&request.at)? {
            return Err(invalid("requested report cursor is not durable"));
        }
        let result = match &request.subject {
            ReportSubject::Execution(id) => {
                execution(
                    &ctx,
                    &self.resources,
                    &TestExecutionId::from_uuid(id.clone())
                        .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
                    &request.at,
                )
                .await
            }
            ReportSubject::Cycle(id) => {
                cycle::materialize(
                    &ctx,
                    &self.resources,
                    &TestCycleId::from_uuid(id.clone())
                        .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
                    &request.at,
                )
                .await
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
    runtime.operations.register_wire_operation::<Handler>(
        "dokime",
        Lanes::Open,
        Arc::new(resources),
    )?;
    Ok(())
}

/* === Private Functions === */
async fn execution(
    ctx: &Context<'_>,
    resources: &Resources,
    id: &TestExecutionId,
    at: &harmos::storage::StorageCursor,
) -> Result<MaterializedReport> {
    let app = ctx.scope().resources::<Dokime>();
    let source = resources
        .execution_storage
        .report_source(id, &app.storage_settings)
        .await?;
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
    let source = resources
        .execution_storage
        .report_source(id, &app.storage_settings)
        .await?;
    let member = source
        .test_executions
        .iter()
        .find(|member| member.test_execution_id == *id)
        .ok_or_else(|| invalid("execution disappeared during report admission"))?;
    if member.saved.is_some() {
        return Err(invalid("sealed execution must read its published report"));
    }
    if let Some(report) = resources
        .execution_storage
        .retry_materialized_report(ctx, id, at)
        .await?
    {
        return Ok(report);
    }
    let has_lifecycle = resources.execution_storage.has_report_lifecycle(id).await?;
    let final_report = has_lifecycle
        && member.finished_at.is_some()
        && member.active_step_attempt.is_none()
        && !matches!(
            member.status,
            dokime::execution::ExecutionStatus::Pending
                | dokime::execution::ExecutionStatus::Running
        );
    if final_report {
        if let Some(reason) = resources.providers.final_report_refusal(member) {
            return Err(Error::InvalidReportInputs(reason));
        }
        resources.execution_storage.finish_outputs(id).await?;
    }
    let inputs = resources
        .execution_storage
        .prepare_capture(&source, member, at, final_report)
        .await?;
    let defaults = resources.settings.read(|settings| settings.report);
    let inspection = resources
        .execution_storage
        .capture_execution_view(&source, &inputs, defaults, &app.storage_settings)
        .await?;
    if inspection.evidence_through()? != at {
        return Err(invalid("capture changed requested report boundary"));
    }
    let progress = resources.progress.begin("Materializing Report");
    let storage = &resources.execution_storage.storage;
    let captured = crate::report::materialize::execution(
        ctx,
        storage,
        &app.storage_settings,
        &inspection,
        final_report,
        Some(&progress),
    )
    .await?
    .try_into_shared()?;
    let outputs =
        inspection.project_outputs(inspection.view().decoded_bytes()?, |outputs, allowance| {
            if outputs.iter().fold(
                std::mem::size_of::<Vec<dokime::execution::saved::ExecutionOutput>>(),
                |bytes, output| bytes.saturating_add(output.memory_size()),
            ) > allowance.get()
            {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "report output metadata",
                });
            }
            Ok(outputs.to_vec())
        })?;
    let renderer = app.renderer;
    let manager = storage.clone();
    let mut prepared =
        tokio::task::spawn_blocking(move || renderer.execution(manager, captured, outputs))
            .await??;
    let result = if final_report {
        resources
            .execution_storage
            .materialize_final_report(ctx, inspection, prepared, Timestamp::now_utc())
            .await
    } else {
        let revision = resources
            .execution_storage
            .publication_revision(inspection.execution())
            .await?;
        prepared.publish_current(ctx, &revision).await
    };
    progress.track(result)
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}
/* === Tests === */

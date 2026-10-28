/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::operations::CallContext;
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

use super::{cycle_id, status};
use crate::grpc::{access::Principal, value::canonical};
use dokime::domain::model::identity::CaseId;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::CorrelationId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::execution as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::command::CommandContext;
use dokime::{RuntimeHandle, transactions::CycleDefinition};

struct Resources {
    runtime: RuntimeHandle,
}

impl Serve<wire::CreateCycle> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::CreateCycle, ctx: Context<'a, wire::CreateCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::CreateCycleReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let mut metadata = request.metadata.take().ok_or_else(|| Status::invalid_argument("cycle metadata is required"))?;
            if metadata.label.trim().is_empty() {
                return Err(Status::invalid_argument("cycle label is required"));
            }
            if metadata.identifier.is_empty() {
                metadata.identifier = format!("cycle.{}", dokime::model::identity::new_id::<CommandId>());
            }
            let id = self
                .runtime
                .create_cycle(
                    CycleDefinition {
                        metadata: canonical::decode_metadata(&metadata)?,
                        continue_automatically: request.continue_automatically,
                    },
                    context,
                )
                .await
                .map_err(status)?;
            Ok((dokime::domain::model::execution::wire::CreateCycleReply {
                cycle_id: id.to_string(),
                observed_cursor: Some(canonical::cursor(&self.runtime.watch_state().latest())),
            })
            .into())
        })
    }
}

impl Serve<wire::CopyCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::CopyCycle, ctx: Context<'a, wire::CopyCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::CreateCycleReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let id = self.runtime.copy_cycle(cycle_id(&request.source_cycle_id)?, context).await.map_err(status)?;
            Ok((dokime::domain::model::execution::wire::CreateCycleReply {
                cycle_id: id.to_string(),
                observed_cursor: Some(canonical::cursor(&self.runtime.watch_state().latest())),
            })
            .into())
        })
    }
}

impl Serve<wire::AppendExecutions> for Resources {
    fn serve<'a>(
        &'a self, request: wire::AppendExecutions, _: Context<'a, wire::AppendExecutions>,
    ) -> Served<'a, dokime::domain::model::execution::wire::AppendCasesReply> {
        Box::pin(async move {
            let cycle = cycle_id(&request.cycle_id)?;
            let executions = request
                .executions
                .iter()
                .map(|binding| {
                    let execution = TestExecutionId::from_uuid(binding.execution_id.clone())
                        .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
                    let case = CaseId::from_wire(binding.case_id.clone()).map_err(|_| Status::invalid_argument("invalid case ID"))?;
                    Ok((execution, case))
                })
                .collect::<Result<Vec<_>, Status>>()?;
            self.runtime.append_executions(cycle, executions).await.map_err(status)?;
            Ok(dokime::domain::model::execution::wire::AppendCasesReply {
                observed_cursor: observed(&self.runtime).observed_cursor,
                execution_ids: request.executions.iter().map(|binding| binding.execution_id.clone()).collect(),
            }
            .into())
        })
    }
}

impl Serve<wire::DeleteCase> for Resources {
    fn serve<'a>(
        &'a self, request: wire::DeleteCase, ctx: Context<'a, wire::DeleteCase>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let cycle = cycle_id(&request.cycle_id)?;
            let case = CaseId::from_wire(request.case_id.clone()).map_err(|_| Status::invalid_argument("invalid case ID"))?;
            self.runtime.delete_cycle_case(cycle, case, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::SetCycleArchived> for Resources {
    fn serve<'a>(
        &'a self, request: wire::SetCycleArchived, ctx: Context<'a, wire::SetCycleArchived>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ArchiveReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            self.runtime.set_cycle_archived(cycle_id(&request.cycle_id)?, request.archived, context).await.map_err(status)?;
            Ok(dokime::domain::model::execution::wire::ArchiveReply { archived: request.archived }.into())
        })
    }
}

impl Serve<wire::SetExecutionArchived> for Resources {
    fn serve<'a>(
        &'a self, request: wire::SetExecutionArchived, ctx: Context<'a, wire::SetExecutionArchived>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ArchiveReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let execution =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
            self.runtime.set_execution_archived(execution, request.archived, context).await.map_err(status)?;
            Ok(dokime::domain::model::execution::wire::ArchiveReply { archived: request.archived }.into())
        })
    }
}

impl Serve<wire::RenameCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::RenameCycle, ctx: Context<'a, wire::RenameCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::RenameReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let label = self.runtime.rename_cycle(cycle_id(&request.cycle_id)?, request.label, context).await.map_err(status)?;
            Ok(dokime::domain::model::execution::wire::RenameReply { label }.into())
        })
    }
}

impl Serve<wire::RenameExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::RenameExecution, ctx: Context<'a, wire::RenameExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::RenameReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let execution =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
            let label = self.runtime.rename_execution(execution, request.label, context).await.map_err(status)?;
            Ok(dokime::domain::model::execution::wire::RenameReply { label }.into())
        })
    }
}

impl Serve<wire::UpdateCyclePolicy> for Resources {
    fn serve<'a>(
        &'a self, request: wire::UpdateCyclePolicy, ctx: Context<'a, wire::UpdateCyclePolicy>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            self.runtime
                .update_cycle_policy(cycle_id(&request.cycle_id)?, request.continue_automatically, context)
                .await
                .map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::PauseCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::PauseCycle, ctx: Context<'a, wire::PauseCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            self.runtime.pause_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::ResumeCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ResumeCycle, ctx: Context<'a, wire::ResumeCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            self.runtime.resume_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::SealCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::SealCycle, ctx: Context<'a, wire::SealCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            self.runtime.seal_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
            Ok(observed(&self.runtime).into())
        })
    }
}

impl Serve<wire::PauseExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::PauseExecution, ctx: Context<'a, wire::PauseExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let id =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            self.runtime.pause_execution(id, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::ResumeExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ResumeExecution, ctx: Context<'a, wire::ResumeExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let id =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            self.runtime.resume_execution(id, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::StopExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::StopExecution, ctx: Context<'a, wire::StopExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let id =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            self.runtime.stop_execution(id, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

impl Serve<wire::CancelExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::CancelExecution, ctx: Context<'a, wire::CancelExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ControlReply> {
        Box::pin(async move {
            let context = command_context(ctx.call(), request.context.as_ref())?;
            let id =
                TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            self.runtime.cancel_execution(id, context).await.map_err(status)?;
            Ok((observed(&self.runtime)).into())
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone() });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::CreateCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::CopyCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::AppendExecutions>::new(served.clone()))?;
    dependencies.insert(Host::<wire::DeleteCase>::new(served.clone()))?;
    dependencies.insert(Host::<wire::SetCycleArchived>::new(served.clone()))?;
    dependencies.insert(Host::<wire::SetExecutionArchived>::new(served.clone()))?;
    dependencies.insert(Host::<wire::RenameCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::RenameExecution>::new(served.clone()))?;
    dependencies.insert(Host::<wire::UpdateCyclePolicy>::new(served.clone()))?;
    dependencies.insert(Host::<wire::PauseCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ResumeCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::SealCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::PauseExecution>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ResumeExecution>::new(served.clone()))?;
    dependencies.insert(Host::<wire::StopExecution>::new(served.clone()))?;
    dependencies.insert(Host::<wire::CancelExecution>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::CreateCycle>()
        .operation::<wire::CopyCycle>()
        .operation::<wire::AppendExecutions>()
        .operation::<wire::DeleteCase>()
        .operation::<wire::SetCycleArchived>()
        .operation::<wire::SetExecutionArchived>()
        .operation::<wire::RenameCycle>()
        .operation::<wire::RenameExecution>()
        .operation::<wire::UpdateCyclePolicy>()
        .operation::<wire::PauseCycle>()
        .operation::<wire::ResumeCycle>()
        .operation::<wire::SealCycle>()
        .operation::<wire::PauseExecution>()
        .operation::<wire::ResumeExecution>()
        .operation::<wire::StopExecution>()
        .operation::<wire::CancelExecution>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(in crate::grpc) fn command_context(
    call: &CallContext, mutation: Option<&dokime::domain::model::execution::wire::MutationContext>,
) -> Result<CommandContext, Status> {
    let correlation_id = mutation
        .and_then(|mutation| mutation.correlation_id.as_ref())
        .map(|id| CorrelationId::from_uuid(id.clone()).map_err(|_| Status::invalid_argument("invalid correlation UUID")))
        .transpose()?;
    // The Configured Shared Authorizer Rejects External Calls Without A Principal.
    // Trusted Internal Calls Retain Native Attribution Without Inventing A Principal.
    let submitted_by = call.get::<Principal>().map(|principal| principal.subject().to_owned());
    Ok(CommandContext {
        correlation_id,
        idempotency_key: None,
        submitted_by,
    })
}

fn observed(runtime: &RuntimeHandle) -> dokime::domain::model::execution::wire::ControlReply {
    dokime::domain::model::execution::wire::ControlReply {
        observed_cursor: Some(canonical::cursor(&runtime.watch_state().latest())),
    }
}

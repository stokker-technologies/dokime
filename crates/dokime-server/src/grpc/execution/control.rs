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
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation};
use tonic::Status;

// Local Imports
use super::{cycle_id, status};
use crate::grpc::{access::Principal, value::canonical};
use dokime::domain::model::identity::CaseId;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::CorrelationId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::operations::execution as wire;
use dokime::runtime::command::CommandContext;
use dokime::{RuntimeHandle, transactions::CycleDefinition};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Create {
    runtime: Arc<RuntimeHandle>,
}

struct Copy {
    runtime: Arc<RuntimeHandle>,
}

struct AppendExecutions {
    runtime: Arc<RuntimeHandle>,
}

struct Delete {
    runtime: Arc<RuntimeHandle>,
}

struct ArchiveCycle {
    runtime: Arc<RuntimeHandle>,
}

struct ArchiveExecution {
    runtime: Arc<RuntimeHandle>,
}

struct RenameCycle {
    runtime: Arc<RuntimeHandle>,
}

struct RenameExecution {
    runtime: Arc<RuntimeHandle>,
}

struct Policy {
    runtime: Arc<RuntimeHandle>,
}

struct PauseCycle {
    runtime: Arc<RuntimeHandle>,
}

struct ResumeCycle {
    runtime: Arc<RuntimeHandle>,
}

struct SealCycle {
    runtime: Arc<RuntimeHandle>,
}

struct PauseExecution {
    runtime: Arc<RuntimeHandle>,
}

struct ResumeExecution {
    runtime: Arc<RuntimeHandle>,
}

struct StopExecution {
    runtime: Arc<RuntimeHandle>,
}

struct CancelExecution {
    runtime: Arc<RuntimeHandle>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum Operations {
    Create(Create),
    Copy(Copy),
    AppendExecutions(AppendExecutions),
    Delete(Delete),
    ArchiveCycle(ArchiveCycle),
    ArchiveExecution(ArchiveExecution),
    RenameCycle(RenameCycle),
    RenameExecution(RenameExecution),
    Policy(Policy),
    PauseCycle(PauseCycle),
    ResumeCycle(ResumeCycle),
    SealCycle(SealCycle),
    PauseExecution(PauseExecution),
    ResumeExecution(ResumeExecution),
    StopExecution(StopExecution),
    CancelExecution(CancelExecution),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Create {
    type Declaration = wire::CreateCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::CreateCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::CreateCycleReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
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
    }
}

impl Operation for Copy {
    type Declaration = wire::CopyCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::CopyCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::CreateCycleReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = self.runtime.copy_cycle(cycle_id(&request.source_cycle_id)?, context).await.map_err(status)?;
        Ok((dokime::domain::model::execution::wire::CreateCycleReply {
            cycle_id: id.to_string(),
            observed_cursor: Some(canonical::cursor(&self.runtime.watch_state().latest())),
        })
        .into())
    }
}

impl Operation for AppendExecutions {
    type Declaration = wire::AppendExecutions;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::AppendExecutions, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::AppendCasesReply>, Status> {
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
    }
}

impl Operation for Delete {
    type Declaration = wire::DeleteCase;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::DeleteCase, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let cycle = cycle_id(&request.cycle_id)?;
        let case = CaseId::from_wire(request.case_id.clone()).map_err(|_| Status::invalid_argument("invalid case ID"))?;
        self.runtime.delete_cycle_case(cycle, case, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ArchiveCycle {
    type Declaration = wire::SetCycleArchived;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::SetCycleArchived, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ArchiveReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime.set_cycle_archived(cycle_id(&request.cycle_id)?, request.archived, context).await.map_err(status)?;
        Ok(dokime::domain::model::execution::wire::ArchiveReply { archived: request.archived }.into())
    }
}

impl Operation for ArchiveExecution {
    type Declaration = wire::SetExecutionArchived;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::SetExecutionArchived, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ArchiveReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let execution =
            TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
        self.runtime.set_execution_archived(execution, request.archived, context).await.map_err(status)?;
        Ok(dokime::domain::model::execution::wire::ArchiveReply { archived: request.archived }.into())
    }
}

impl Operation for RenameCycle {
    type Declaration = wire::RenameCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::RenameCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::RenameReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let label =
            self.runtime.rename_cycle(cycle_id(&request.cycle_id)?, std::mem::take(&mut request.label), context).await.map_err(status)?;
        Ok(dokime::domain::model::execution::wire::RenameReply { label }.into())
    }
}

impl Operation for RenameExecution {
    type Declaration = wire::RenameExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::RenameExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::RenameReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let execution =
            TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
        let label = self.runtime.rename_execution(execution, std::mem::take(&mut request.label), context).await.map_err(status)?;
        Ok(dokime::domain::model::execution::wire::RenameReply { label }.into())
    }
}

impl Operation for Policy {
    type Declaration = wire::UpdateCyclePolicy;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::UpdateCyclePolicy, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime.update_cycle_policy(cycle_id(&request.cycle_id)?, request.continue_automatically, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for PauseCycle {
    type Declaration = wire::PauseCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::PauseCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime.pause_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ResumeCycle {
    type Declaration = wire::ResumeCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::ResumeCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime.resume_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for SealCycle {
    type Declaration = wire::SealCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::SealCycle, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime.seal_cycle(cycle_id(&request.cycle_id)?, context).await.map_err(status)?;
        Ok(observed(&self.runtime).into())
    }
}

impl Operation for PauseExecution {
    type Declaration = wire::PauseExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::PauseExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime.pause_execution(id, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ResumeExecution {
    type Declaration = wire::ResumeExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::ResumeExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime.resume_execution(id, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for StopExecution {
    type Declaration = wire::StopExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::StopExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime.stop_execution(id, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for CancelExecution {
    type Declaration = wire::CancelExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::CancelExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime.cancel_execution(id, context).await.map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operations::<Operations, RuntimeHandle>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(runtime.clone()),
    )?;
    Ok(())
}

pub(in crate::grpc) fn command_context(
    context: &Context<'_>, mutation: Option<&dokime::domain::model::execution::wire::MutationContext>,
) -> Result<CommandContext, Status> {
    let correlation_id = mutation
        .and_then(|mutation| mutation.correlation_id.as_ref())
        .map(|id| CorrelationId::from_uuid(id.clone()).map_err(|_| Status::invalid_argument("invalid correlation UUID")))
        .transpose()?;
    // The Configured Shared Authorizer Rejects External Calls Without A Principal.
    // Trusted Internal Calls Retain Native Attribution Without Inventing A Principal.
    let submitted_by = context.context().get::<Principal>().map(|principal| principal.subject().to_owned());
    Ok(CommandContext { correlation_id, idempotency_key: None, submitted_by })
}

/* === Private Functions === */

fn observed(runtime: &RuntimeHandle) -> dokime::domain::model::execution::wire::ControlReply {
    dokime::domain::model::execution::wire::ControlReply { observed_cursor: Some(canonical::cursor(&runtime.watch_state().latest())) }
}

/* === Tests === */

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
use harmos::operations::{Context, Operation};
use tonic::Status;

// Local Imports
use super::{cycle_id, status};
use crate::grpc::{
    access::Principal,
    planning,
    value::{self, canonical},
};
use dokime_app::core::{CaseId, CommandId, CorrelationId, TestExecutionId};
use dokime_app::runtime::command::CommandContext;
use dokime_app::{RuntimeHandle, transactions::CycleDefinition};
use dokime_contract::operations::execution as wire;

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

struct Append {
    runtime: Arc<RuntimeHandle>,
}

struct Delete {
    runtime: Arc<RuntimeHandle>,
}

struct Rerun {
    runtime: Arc<RuntimeHandle>,
}

struct ArchiveCycle {
    runtime: Arc<RuntimeHandle>,
}

struct ArchiveExecution {
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

struct StopCycle {
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

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Create(Create),
    Copy(Copy),
    Append(Append),
    Delete(Delete),
    Rerun(Rerun),
    ArchiveCycle(ArchiveCycle),
    ArchiveExecution(ArchiveExecution),
    Policy(Policy),
    PauseCycle(PauseCycle),
    ResumeCycle(ResumeCycle),
    StopCycle(StopCycle),
    PauseExecution(PauseExecution),
    ResumeExecution(ResumeExecution),
    StopExecution(StopExecution),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Create {
    type Declaration = wire::CreateCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CreateCycleRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::CreateCycleReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let snapshot = self.runtime.watch_state().latest();
        let plan = planning::resolve(
            &snapshot.state().suites,
            request.plan.take(),
            &CommandId::new(),
        )?;
        let metadata = request
            .metadata
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("cycle metadata is required"))?;
        value::text(&metadata.label)?;
        if metadata.label.trim().is_empty() {
            return Err(Status::invalid_argument("cycle label is required"));
        }
        let metadata = canonical::decode_metadata(metadata)?;
        let id = self
            .runtime
            .start_cycle(
                CycleDefinition {
                    suite: plan.suite,
                    metadata,
                    cases: plan.cases,
                    continue_on_failure: request.continue_on_failure,
                },
                context,
            )
            .await
            .map_err(status)?;
        Ok((wire::CreateCycleReply {
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

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CopyCycleRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::CreateCycleReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = self
            .runtime
            .copy_cycle(cycle_id(&request.source_cycle_id)?, context)
            .await
            .map_err(status)?;
        Ok((wire::CreateCycleReply {
            cycle_id: id.to_string(),
            observed_cursor: Some(canonical::cursor(&self.runtime.watch_state().latest())),
        })
        .into())
    }
}

impl Operation for Append {
    type Declaration = wire::AppendPlannedCases;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::AppendPlannedCasesRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let plan = planning::resolve(
            &snapshot.state().suites,
            request.plan.take(),
            &CommandId::new(),
        )?;
        self.runtime
            .append_cycle_templates(id, plan.suite, plan.cases, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for Delete {
    type Declaration = wire::DeleteCase;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CaseControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let cycle = cycle_id(&request.cycle_id)?;
        value::text(&request.case_id)?;
        let case = CaseId::from_wire(request.case_id.clone())
            .map_err(|_| Status::invalid_argument("invalid case ID"))?;
        self.runtime
            .delete_cycle_case(cycle, case, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for Rerun {
    type Declaration = wire::ScheduleCaseRerun;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CaseControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        let cycle = cycle_id(&request.cycle_id)?;
        value::text(&request.case_id)?;
        let case = CaseId::from_wire(request.case_id.clone())
            .map_err(|_| Status::invalid_argument("invalid case ID"))?;
        self.runtime
            .rerun_cycle_case(cycle, case, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ArchiveCycle {
    type Declaration = wire::SetCycleArchived;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::SetCycleArchivedRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ArchiveReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime
            .set_cycle_archived(cycle_id(&request.cycle_id)?, request.archived, context)
            .await
            .map_err(status)?;
        Ok(wire::ArchiveReply {
            archived: request.archived,
        }
        .into())
    }
}

impl Operation for ArchiveExecution {
    type Declaration = wire::SetExecutionArchived;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::SetExecutionArchivedRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ArchiveReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        value::text(&request.execution_id)?;
        let execution = TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
        self.runtime
            .set_execution_archived(execution, request.archived, context)
            .await
            .map_err(status)?;
        Ok(wire::ArchiveReply {
            archived: request.archived,
        }
        .into())
    }
}

impl Operation for Policy {
    type Declaration = wire::UpdateCyclePolicy;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::UpdateCyclePolicyRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime
            .update_cycle_policy(
                cycle_id(&request.cycle_id)?,
                request.continue_on_failure,
                context,
            )
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for PauseCycle {
    type Declaration = wire::PauseCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CycleControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime
            .pause_cycle(cycle_id(&request.cycle_id)?, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ResumeCycle {
    type Declaration = wire::ResumeCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CycleControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime
            .resume_cycle(cycle_id(&request.cycle_id)?, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for StopCycle {
    type Declaration = wire::StopCycle;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::CycleControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        self.runtime
            .stop_cycle(cycle_id(&request.cycle_id)?, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for PauseExecution {
    type Declaration = wire::PauseExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::ExecutionControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        value::text(&request.execution_id)?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime
            .pause_execution(id, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for ResumeExecution {
    type Declaration = wire::ResumeExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::ExecutionControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        value::text(&request.execution_id)?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime
            .resume_execution(id, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

impl Operation for StopExecution {
    type Declaration = wire::StopExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        runtime: Arc<RuntimeHandle>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::ExecutionControlRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ControlReply>, Status> {
        let context = command_context(&context, request.context.as_ref())?;
        value::text(&request.execution_id)?;
        let id = TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime
            .stop_execution(id, context)
            .await
            .map_err(status)?;
        Ok((observed(&self.runtime)).into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, RuntimeHandle>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(runtime.clone()),
        )?;
    Ok(())
}

pub(in crate::grpc) fn command_context(
    context: &Context<'_>,
    mutation: Option<&wire::MutationContext>,
) -> Result<CommandContext, Status> {
    let correlation_id = mutation
        .and_then(|mutation| mutation.correlation_id.as_ref())
        .map(|id| {
            CorrelationId::from_uuid(id.clone())
                .map_err(|_| Status::invalid_argument("invalid correlation UUID"))
        })
        .transpose()?;
    // The Configured Shared Authorizer Rejects External Calls Without A Principal.
    // Trusted Internal Calls Retain Native Attribution Without Inventing A Principal.
    let submitted_by = context
        .context()
        .get::<Principal>()
        .map(|principal| principal.subject().to_owned());
    Ok(CommandContext {
        correlation_id,
        idempotency_key: None,
        submitted_by,
    })
}

/* === Private Functions === */
fn observed(runtime: &RuntimeHandle) -> wire::ControlReply {
    wire::ControlReply {
        observed_cursor: Some(canonical::cursor(&runtime.watch_state().latest())),
    }
}

/* === Tests === */

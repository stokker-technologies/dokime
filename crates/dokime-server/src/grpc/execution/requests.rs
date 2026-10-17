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

// Local Imports
use super::loan;
use super::*;
use crate::grpc::access::{Principal, Role};
use dokime_runtime::runtime::command::{CommandContext, CommandReceipt, CommandStatus};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::CommandId;
use dokime_runtime::domains::execution::requests::RunPhase;
use dokime_runtime::domains::execution::requests::RunReceipt;
use dokime_domain::model::identity::wire as common;
use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::operations::execution as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Lookup {
    resources: Arc<super::Resources>,
}

struct Cycle {
    runtime: Arc<RuntimeHandle>,
}
struct Case {
    runtime: Arc<RuntimeHandle>,
}
struct Execution {
    runtime: Arc<RuntimeHandle>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Cycle(Cycle),
    Case(Case),
    Execution(Execution),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Cycle {
    type Declaration = wire::RunCycle;
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
        request: &mut wire::RunCycle,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::RunReply>, Status> {
        let context = context_for_run(
            &context,
            request.context.as_ref(),
            request.idempotency_key.as_deref(),
        )?;
        let id = cycle_id(&request.cycle_id)?;
        let result = self.runtime.run_cycle_cases(id, context).await;
        receipt(result, self.runtime.process_epoch().to_string()).map(Into::into)
    }
}

impl Operation for Case {
    type Declaration = wire::RunCase;
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
        request: &mut wire::RunCase,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::RunReply>, Status> {
        let context = context_for_run(
            &context,
            request.context.as_ref(),
            request.idempotency_key.as_deref(),
        )?;
        let id = cycle_id(&request.cycle_id)?;
        value::text(&request.case_id)?;
        let case = CaseId::from_wire(request.case_id.clone())
            .map_err(|_| Status::invalid_argument("invalid case ID"))?;
        let result = self.runtime.run_cycle_case(id, case, context).await;
        receipt(result, self.runtime.process_epoch().to_string()).map(Into::into)
    }
}

impl Operation for Execution {
    type Declaration = wire::RunExecution;
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
        request: &mut wire::RunExecution,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::RunReply>, Status> {
        let context = context_for_run(
            &context,
            request.context.as_ref(),
            request.idempotency_key.as_deref(),
        )?;
        value::text(&request.execution_id)?;
        let id = dokime_domain::model::identity::TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        let mode = match request.mode {
            dokime_domain::model::execution::wire::ExecutionMode::Automatic => dokime_domain::model::execution::ExecutionMode::Automatic,
            dokime_domain::model::execution::wire::ExecutionMode::Interactive => dokime_domain::model::execution::ExecutionMode::Interactive,
        };
        let result = self.runtime.run_execution(id, mode, context).await;
        receipt(result, self.runtime.process_epoch().to_string()).map(Into::into)
    }
}

impl Operation for Lookup {
    type Declaration = wire::GetRunRequest;
    type Resources = super::Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Self::Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::GetRunRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::RunRequestReceipt>, Status> {
        get(&self.resources, principal(&context)?, request).await
    }
}

/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, RuntimeHandle>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            Arc::new(runtime.clone()),
        )?;
    runtime.operations().register_wire_operation::<Lookup>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(super::Resources {
            runtime: runtime.clone(),
            max_message_bytes,
        }),
    )?;
    Ok(())
}

async fn get(
    resources: &super::Resources,
    principal: &Principal,
    request: &wire::GetRunRequest,
) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::RunRequestReceipt>, Status> {
    let runtime = &resources.runtime;
    value::text(&request.request_id)?;
    let id = CommandId::from_uuid(request.request_id.clone())
        .map_err(|_| Status::invalid_argument("invalid run request UUID"))?;
    let receipt = runtime
        .run_request(&id)
        .await
        .map_err(|error| {
            super::super::error::detailed(
                Status::unavailable("cannot resolve primary run receipt; retry lookup"),
                &error,
            )
        })?
        .ok_or_else(|| Status::not_found("run request not found"))?;
    authorize(principal, &receipt)?;
    let output = project(&receipt)?;
    loan::response_limit(&output, resources.max_message_bytes)?;
    Ok(output.into())
}

pub(super) fn authorize(principal: &Principal, receipt: &RunReceipt) -> Result<(), Status> {
    if principal.role() != Role::Admin
        && receipt.submitted_by.as_deref() != Some(principal.subject())
    {
        return Err(Status::not_found("run request not found"));
    }
    Ok(())
}

pub(super) fn principal<'a>(context: &'a Context<'_>) -> Result<&'a Principal, Status> {
    context
        .context()
        .get::<Principal>()
        .ok_or_else(|| Status::unauthenticated("authentication required"))
}

/* === Private Functions === */
fn context_for_run(
    context: &Context<'_>,
    labels: Option<&dokime_domain::model::execution::wire::MutationContext>,
    key: Option<&str>,
) -> Result<CommandContext, Status> {
    let mut context = super::control::command_context(context, labels)?;
    if let Some(key) = key {
        dokime_domain::model::execution::command::validate_idempotency_key(key)
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        context.idempotency_key = Some(key.to_owned());
    }
    Ok(context)
}

fn receipt(receipt: CommandReceipt, epoch: String) -> Result<dokime_domain::model::execution::wire::RunReply, Status> {
    let outcome = match receipt.status {
        CommandStatus::Rejected => {
            let detail = receipt
                .rejection
                .ok_or_else(|| Status::internal("rejected receipt omitted reason"))?;
            dokime_domain::model::execution::wire::RunReplyOutcome::Rejected(super::rejection(
                detail.code,
                detail.message,
                detail.validation_issues,
            ))
        }
        CommandStatus::Accepted => {
            let at = time::OffsetDateTime::parse(
                receipt.accepted_at.as_str(),
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| Status::internal("invalid admission timestamp"))?;
            dokime_domain::model::execution::wire::RunReplyOutcome::Accepted(dokime_domain::model::execution::wire::Admission {
                command_id: receipt.command_id.to_string(),
                job_id: receipt.job_id.map(|id| id.to_string()),
                cursor: Some(Cursor {
                    epoch,
                    stream: crate::grpc::observation::STREAM.into(),
                    position: receipt.runtime_revision,
                }),
                accepted_at: Some(common::Timestamp {
                    seconds: at.unix_timestamp(),
                    nanos: at.nanosecond() as i32,
                }),
            })
        }
    };
    Ok(dokime_domain::model::execution::wire::RunReply {
        outcome: Some(outcome),
    })
}

fn project(receipt: &RunReceipt) -> Result<dokime_domain::model::execution::wire::RunRequestReceipt, Status> {
    let summary = &receipt.summary;
    if let Some(interruption) = &receipt.interruption {
        value::text(interruption)?;
    }
    Ok(dokime_domain::model::execution::wire::RunRequestReceipt {
        request_id: summary.request_id.as_str().to_owned(),
        mode: match summary.mode {
            dokime_domain::model::execution::ExecutionMode::Automatic => dokime_domain::model::execution::wire::ExecutionMode::Automatic,
            dokime_domain::model::execution::ExecutionMode::Interactive => dokime_domain::model::execution::wire::ExecutionMode::Interactive,
        },
        node_id: receipt.node_id.as_str().to_owned(),
        cycle_id: summary.cycle_id.as_str().to_owned(),
        phase: match summary.phase {
            RunPhase::Queued => dokime_domain::model::execution::wire::RunRequestPhase::Queued,
            RunPhase::Preparing => dokime_domain::model::execution::wire::RunRequestPhase::Preparing,
            RunPhase::Running => dokime_domain::model::execution::wire::RunRequestPhase::Running,
            RunPhase::Finishing => dokime_domain::model::execution::wire::RunRequestPhase::Finishing,
            RunPhase::Completed => dokime_domain::model::execution::wire::RunRequestPhase::Completed,
            RunPhase::Failed => dokime_domain::model::execution::wire::RunRequestPhase::Failed,
            RunPhase::Stopped => dokime_domain::model::execution::wire::RunRequestPhase::Stopped,
            RunPhase::Interrupted => dokime_domain::model::execution::wire::RunRequestPhase::Interrupted,
        },
        paused: summary.paused,
        stop_requested: summary.stop_requested,
        planned_count: summary.planned,
        prepared_count: summary.prepared,
        finished_count: summary.finished,
        unstarted_count: summary.unstarted,
        failed_count: summary.failed,
        claimed_count: receipt.claimed,
        last_claim: receipt
            .last_claim
            .as_ref()
            .map(|claim| dokime_domain::model::execution::wire::RunAttemptClaim {
                ordinal: claim.ordinal,
                case_id: claim.case_id.as_str().to_owned(),
                execution_id: claim.execution_id.as_str().to_owned(),
            }),
        accepted_at: Some(canonical::timestamp(&summary.accepted_at)?),
        provider_started_at: summary
            .provider_started_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        finished_at: summary
            .finished_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        submitted_by: receipt.submitted_by.clone(),
        interruption: receipt.interruption.clone(),
    })
}

/* === Tests === */

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

// Third Party Imports
use std::sync::Arc;

use harmos::operations::{Context, Operation};
use tonic::Status;
// Local Imports
use super::loan;
use super::{requests, value};
use crate::grpc::access::{Principal, Role};
use crate::grpc::value::canonical;
use dokime_app::core::{CommandId, StepId, TestExecutionId};
use dokime_app::domains::execution::requests::{
    RunReceipt, StepCommand, StepCommandPhase, StepCommandReceipt,
};
use dokime_app::{Error, RuntimeHandle};
use dokime_contract::operations::execution as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Submit {
    resources: Arc<super::Resources>,
}
struct Lookup {
    resources: Arc<super::Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Submit(Submit),
    Lookup(Lookup),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Submit {
    type Declaration = wire::SubmitExecutionCommand;
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
        request: &mut wire::SubmitExecutionCommandRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ExecutionCommandReceipt>, Status> {
        submit(&self.resources, requests::principal(&context)?, request).await
    }
}

impl Operation for Lookup {
    type Declaration = wire::GetExecutionCommand;
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
        request: &mut wire::GetExecutionCommandRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ExecutionCommandReceipt>, Status> {
        get(&self.resources, requests::principal(&context)?, request).await
    }
}

/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, super::Resources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(super::Resources {
                runtime: runtime.clone(),
                max_message_bytes,
            }),
        )?;
    Ok(())
}

async fn submit(
    resources: &super::Resources,
    principal: &Principal,
    request: &wire::SubmitExecutionCommandRequest,
) -> Result<harmos::operations::Response<wire::ExecutionCommandReceipt>, Status> {
    let runtime = &resources.runtime;
    if principal.role() < Role::Control {
        return Err(Status::permission_denied("control permission is required"));
    }
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let execution_id = TestExecutionId::from_uuid(request.execution_id.clone())
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let action = action(request.action, request.step_id.as_deref())?;
    owned(runtime, principal, &request_id).await?;
    let receipt = runtime
        .admit_execution_command(&request_id, command_id, execution_id, action)
        .await
        .map_err(|error| match error {
            Error::Conflict { message, .. } => Status::failed_precondition(message),
            error => super::super::error::detailed(
                Status::unavailable(
                    "command admission is uncertain; resolve the same command UUID",
                ),
                &error,
            ),
        })?;
    charged_project(resources, &request_id, &receipt)
}

async fn get(
    resources: &super::Resources,
    principal: &Principal,
    request: &wire::GetExecutionCommandRequest,
) -> Result<harmos::operations::Response<wire::ExecutionCommandReceipt>, Status> {
    let runtime = &resources.runtime;
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let receipt = owned(runtime, principal, &request_id).await?;
    let command = receipt
        .commands
        .get(&command_id)
        .ok_or_else(|| Status::not_found("execution command not found"))?;
    charged_project(resources, &request_id, command)
}

/* === Private Functions === */
async fn owned(
    runtime: &RuntimeHandle,
    principal: &Principal,
    request_id: &CommandId,
) -> Result<harmos::storage::SharedQueryValue<RunReceipt>, Status> {
    let receipt = runtime
        .run_request(request_id)
        .await
        .map_err(|error| {
            super::super::error::detailed(
                Status::unavailable("cannot resolve primary run receipt; retry lookup"),
                &error,
            )
        })?
        .ok_or_else(|| Status::not_found("run request not found"))?;
    requests::authorize(principal, &receipt)?;
    Ok(receipt)
}

fn command_id(id: &str) -> Result<CommandId, Status> {
    value::text(id)?;
    CommandId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid command UUID"))
}

fn action(
    action: wire::ExecutionCommandAction,
    step_id: Option<&str>,
) -> Result<StepCommand, Status> {
    match (action, step_id) {
        (wire::ExecutionCommandAction::Next, None) => Ok(StepCommand::Next),
        (wire::ExecutionCommandAction::Finish, None) => Ok(StepCommand::Finish),
        (wire::ExecutionCommandAction::RunStep, Some(id)) => {
            value::text(id)?;
            let step_id = StepId::from_uuid(id)
                .map_err(|_| Status::invalid_argument("invalid frozen step UUID"))?;
            Ok(StepCommand::RunStep { step_id })
        }
        _ => Err(Status::invalid_argument(
            "command action and step UUID do not match",
        )),
    }
}

fn charged_project(
    resources: &super::Resources,
    request_id: &CommandId,
    receipt: &StepCommandReceipt,
) -> Result<harmos::operations::Response<wire::ExecutionCommandReceipt>, Status> {
    let bytes = std::mem::size_of::<wire::ExecutionCommandReceipt>()
        .saturating_add(request_id.as_str().len())
        .saturating_add(receipt.command_id.as_str().len())
        .saturating_add(receipt.execution_id.as_str().len())
        .saturating_add(
            receipt
                .attempt_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(match &receipt.action {
            StepCommand::RunStep { step_id } => step_id.as_str().len(),
            StepCommand::Next | StepCommand::Finish => 0,
        })
        .saturating_add(receipt.detail.as_ref().map_or(0, String::len));
    let preparation = loan::reserve(&resources.runtime, bytes)?;
    let output = project(request_id, receipt)?;
    loan::response_limit(&output, resources.max_message_bytes)?;
    preparation
        .finish(output)
        .map_err(loan::memory_error)?
        .try_into_value()
        .map_err(loan::memory_error)?
        .try_into_response()
        .map_err(loan::memory_error)
}

fn project(
    request_id: &CommandId,
    receipt: &StepCommandReceipt,
) -> Result<wire::ExecutionCommandReceipt, Status> {
    let (action, step_id) = match &receipt.action {
        StepCommand::Next => (wire::ExecutionCommandAction::Next, None),
        StepCommand::RunStep { step_id } => (
            wire::ExecutionCommandAction::RunStep,
            Some(step_id.as_str().to_owned()),
        ),
        StepCommand::Finish => (wire::ExecutionCommandAction::Finish, None),
    };
    if let Some(detail) = &receipt.detail {
        value::text(detail)?;
    }
    Ok(wire::ExecutionCommandReceipt {
        request_id: request_id.as_str().to_owned(),
        command_id: receipt.command_id.as_str().to_owned(),
        execution_id: receipt.execution_id.as_str().to_owned(),
        action,
        step_id,
        phase: match receipt.phase {
            StepCommandPhase::Accepted => wire::ExecutionCommandPhase::Accepted,
            StepCommandPhase::Dispatching => wire::ExecutionCommandPhase::Dispatching,
            StepCommandPhase::Running => wire::ExecutionCommandPhase::Running,
            StepCommandPhase::Completed => wire::ExecutionCommandPhase::Completed,
            StepCommandPhase::Refused => wire::ExecutionCommandPhase::Refused,
            StepCommandPhase::Failed => wire::ExecutionCommandPhase::Failed,
            StepCommandPhase::Interrupted => wire::ExecutionCommandPhase::Interrupted,
        },
        attempt_id: receipt.attempt_id.as_ref().map(|id| id.as_str().to_owned()),
        accepted_at: Some(canonical::timestamp(&receipt.accepted_at)?),
        updated_at: Some(canonical::timestamp(&receipt.updated_at)?),
        detail: receipt.detail.clone(),
    })
}

/* === Tests === */

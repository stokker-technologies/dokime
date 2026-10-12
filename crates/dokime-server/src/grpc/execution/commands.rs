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
use tonic::{Request, Status};
// Local Imports
use super::codec::{self, CommandWire};
use super::{Principal, Role, requests, value};
use dokime_app::capabilities::execution::requests::{
    RunReceipt, StepCommand, StepCommandPhase, StepCommandReceipt,
};
use dokime_app::core::{CommandId, StepId, TestExecutionId};
use dokime_app::{Error, RuntimeHandle};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn submit(
    runtime: &RuntimeHandle,
    request: Request<v1::SubmitExecutionCommandRequest>,
) -> Result<codec::CommandReceipt, Status> {
    let principal = Principal::of(&request)?.clone();
    if principal.role() < Role::Control {
        return Err(Status::permission_denied("control permission is required"));
    }
    let request = request.into_inner();
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let execution_id = TestExecutionId::from_uuid(request.execution_id)
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let action = action(request.action, request.step_id)?;
    owned(runtime, &principal, &request_id).await?;
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
    charged_project(runtime, &request_id, &receipt)
}

pub(super) async fn get(
    runtime: &RuntimeHandle,
    request: Request<v1::GetExecutionCommandRequest>,
) -> Result<codec::CommandReceipt, Status> {
    let principal = Principal::of(&request)?.clone();
    let request = request.into_inner();
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let receipt = owned(runtime, &principal, &request_id).await?;
    let command = receipt
        .commands
        .get(&command_id)
        .ok_or_else(|| Status::not_found("execution command not found"))?;
    charged_project(runtime, &request_id, command)
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
    CommandId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid command UUID"))
}

fn action(action: i32, step_id: Option<String>) -> Result<StepCommand, Status> {
    match (v1::ExecutionCommandAction::try_from(action), step_id) {
        (Ok(v1::ExecutionCommandAction::Next), None) => Ok(StepCommand::Next),
        (Ok(v1::ExecutionCommandAction::Finish), None) => Ok(StepCommand::Finish),
        (Ok(v1::ExecutionCommandAction::RunStep), Some(id)) => {
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
    runtime: &RuntimeHandle,
    request_id: &CommandId,
    receipt: &StepCommandReceipt,
) -> Result<codec::CommandReceipt, Status> {
    let bytes = std::mem::size_of::<CommandWire>()
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
    let preparation = codec::reserve(runtime, bytes)?;
    let output = CommandWire(project(request_id, receipt)?);
    preparation
        .finish(output)
        .map_err(codec::memory_error)?
        .try_into_value()
        .map_err(codec::memory_error)
}

fn project(
    request_id: &CommandId,
    receipt: &StepCommandReceipt,
) -> Result<v1::ExecutionCommandReceipt, Status> {
    let (action, step_id) = match &receipt.action {
        StepCommand::Next => (v1::ExecutionCommandAction::Next, None),
        StepCommand::RunStep { step_id } => (
            v1::ExecutionCommandAction::RunStep,
            Some(step_id.as_str().to_owned()),
        ),
        StepCommand::Finish => (v1::ExecutionCommandAction::Finish, None),
    };
    if let Some(detail) = &receipt.detail {
        value::text(detail)?;
    }
    Ok(v1::ExecutionCommandReceipt {
        request_id: request_id.as_str().to_owned(),
        command_id: receipt.command_id.as_str().to_owned(),
        execution_id: receipt.execution_id.as_str().to_owned(),
        action: action as i32,
        step_id,
        phase: match receipt.phase {
            StepCommandPhase::Accepted => v1::ExecutionCommandPhase::Accepted,
            StepCommandPhase::Dispatching => v1::ExecutionCommandPhase::Dispatching,
            StepCommandPhase::Running => v1::ExecutionCommandPhase::Running,
            StepCommandPhase::Completed => v1::ExecutionCommandPhase::Completed,
            StepCommandPhase::Refused => v1::ExecutionCommandPhase::Refused,
            StepCommandPhase::Failed => v1::ExecutionCommandPhase::Failed,
            StepCommandPhase::Interrupted => v1::ExecutionCommandPhase::Interrupted,
        } as i32,
        attempt_id: receipt.attempt_id.as_ref().map(|id| id.as_str().to_owned()),
        accepted_at: Some(requests::timestamp(&receipt.accepted_at)?),
        updated_at: Some(requests::timestamp(&receipt.updated_at)?),
        detail: receipt.detail.clone(),
    })
}

/* === Tests === */

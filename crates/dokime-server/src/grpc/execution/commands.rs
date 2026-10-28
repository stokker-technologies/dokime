/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

use super::loan;
use crate::grpc::value::canonical;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::StepId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domains::execution::requests::{Receipt, StepCommand, StepCommandPhase, StepCommandReceipt};
use dokime::operations::execution as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::{RuntimeHandle, runtime::Error};

impl Serve<wire::SubmitExecutionCommand> for super::Resources {
    fn serve<'a>(
        &'a self, request: wire::SubmitExecutionCommand, _: Context<'a, wire::SubmitExecutionCommand>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ExecutionCommandReceipt> {
        Box::pin(async move { submit(self, &request).await })
    }
}

impl Serve<wire::GetExecutionCommand> for super::Resources {
    fn serve<'a>(
        &'a self, request: wire::GetExecutionCommand, _: Context<'a, wire::GetExecutionCommand>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ExecutionCommandReceipt> {
        Box::pin(async move { get(self, &request).await })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(super::Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::SubmitExecutionCommand>::new(served.clone()))?;
    dependencies.insert(Host::<wire::GetExecutionCommand>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::SubmitExecutionCommand>()
        .operation::<wire::GetExecutionCommand>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

async fn submit(
    resources: &super::Resources, request: &wire::SubmitExecutionCommand,
) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ExecutionCommandReceipt>, Status> {
    let runtime = &resources.runtime;
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let execution_id =
        TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let action = action(request.action, request.step_id.as_deref())?;
    receipt(runtime, &request_id).await?;
    let receipt = runtime.admit_execution_command(&request_id, command_id, execution_id, action).await.map_err(|error| match error {
        Error::Admission(AdmissionFailure::Conflict { message, .. }) => Status::failed_precondition(message),
        error => {
            super::super::error::detailed(Status::unavailable("command admission is uncertain; resolve the same command UUID"), &error)
        }
    })?;
    charged_project(resources, &request_id, &receipt)
}

async fn get(
    resources: &super::Resources, request: &wire::GetExecutionCommand,
) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ExecutionCommandReceipt>, Status> {
    let runtime = &resources.runtime;
    let request_id = command_id(&request.request_id)?;
    let command_id = command_id(&request.command_id)?;
    let receipt = receipt(runtime, &request_id).await?;
    let command = receipt.commands.get(&command_id).ok_or_else(|| Status::not_found("execution command not found"))?;
    charged_project(resources, &request_id, command)
}

async fn receipt(runtime: &RuntimeHandle, request_id: &CommandId) -> Result<Receipt, Status> {
    let receipt = runtime
        .run_request(request_id)
        .await
        .map_err(|error| super::super::error::detailed(Status::unavailable("cannot resolve primary run receipt; retry lookup"), &error))?
        .ok_or_else(|| Status::not_found("run request not found"))?;
    Ok(receipt)
}

fn command_id(id: &str) -> Result<CommandId, Status> {
    CommandId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid command UUID"))
}

fn action(action: dokime::domain::model::execution::wire::ExecutionCommandAction, step_id: Option<&str>) -> Result<StepCommand, Status> {
    match (action, step_id) {
        (dokime::domain::model::execution::wire::ExecutionCommandAction::Next, None) => Ok(StepCommand::Next),
        (dokime::domain::model::execution::wire::ExecutionCommandAction::Finish, None) => Ok(StepCommand::Finish),
        (dokime::domain::model::execution::wire::ExecutionCommandAction::RunStep, Some(id)) => {
            let step_id = StepId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid frozen step UUID"))?;
            Ok(StepCommand::RunStep { step_id })
        }
        _ => Err(Status::invalid_argument("command action and step UUID do not match")),
    }
}

fn charged_project(
    resources: &super::Resources, request_id: &CommandId, receipt: &StepCommandReceipt,
) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ExecutionCommandReceipt>, Status> {
    let output = project(request_id, receipt)?;
    loan::response_limit(&output, resources.max_message_bytes)?;
    Ok(output.into())
}

fn project(
    request_id: &CommandId, receipt: &StepCommandReceipt,
) -> Result<dokime::domain::model::execution::wire::ExecutionCommandReceipt, Status> {
    let (action, step_id) = match &receipt.action {
        StepCommand::Next => (dokime::domain::model::execution::wire::ExecutionCommandAction::Next, None),
        StepCommand::RunStep { step_id } => {
            (dokime::domain::model::execution::wire::ExecutionCommandAction::RunStep, Some(step_id.as_str().to_owned()))
        }
        StepCommand::Finish => (dokime::domain::model::execution::wire::ExecutionCommandAction::Finish, None),
    };
    Ok(dokime::domain::model::execution::wire::ExecutionCommandReceipt {
        request_id: request_id.as_str().to_owned(),
        command_id: receipt.command_id.as_str().to_owned(),
        execution_id: receipt.execution_id.as_str().to_owned(),
        action,
        step_id,
        phase: match receipt.phase {
            StepCommandPhase::Accepted => dokime::domain::model::execution::wire::ExecutionCommandPhase::Accepted,
            StepCommandPhase::Dispatching => dokime::domain::model::execution::wire::ExecutionCommandPhase::Dispatching,
            StepCommandPhase::Running => dokime::domain::model::execution::wire::ExecutionCommandPhase::Running,
            StepCommandPhase::Completed => dokime::domain::model::execution::wire::ExecutionCommandPhase::Completed,
            StepCommandPhase::Refused => dokime::domain::model::execution::wire::ExecutionCommandPhase::Refused,
            StepCommandPhase::Failed => dokime::domain::model::execution::wire::ExecutionCommandPhase::Failed,
            StepCommandPhase::Interrupted => dokime::domain::model::execution::wire::ExecutionCommandPhase::Interrupted,
        },
        attempt_id: receipt.attempt_id.as_ref().map(|id| id.as_str().to_owned()),
        accepted_at: Some(canonical::timestamp(&receipt.accepted_at)?),
        updated_at: Some(canonical::timestamp(&receipt.updated_at)?),
        detail: receipt.detail.clone(),
    })
}

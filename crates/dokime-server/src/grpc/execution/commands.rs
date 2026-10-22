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
use super::loan;
use super::value;
use crate::grpc::value::canonical;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::StepId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::operations::execution as wire;
use dokime::domains::execution::requests::{RunReceipt, StepCommand, StepCommandPhase, StepCommandReceipt};
use dokime::{RuntimeHandle, runtime::Error};

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

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
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
    async fn initialize<'ctx>(resources: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::SubmitExecutionCommand, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ExecutionCommandReceipt>, Status> {
        submit(&self.resources, request).await
    }
}

impl Operation for Lookup {
    type Declaration = wire::GetExecutionCommand;
    type Resources = super::Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetExecutionCommand, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::ExecutionCommandReceipt>, Status> {
        get(&self.resources, request).await
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operations::<Operations, super::Resources>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(super::Resources { runtime: runtime.clone(), max_message_bytes }),
    )?;
    Ok(())
}

/* === Private Functions === */

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

async fn receipt(runtime: &RuntimeHandle, request_id: &CommandId) -> Result<std::sync::Arc<RunReceipt>, Status> {
    let receipt = runtime
        .run_request(request_id)
        .await
        .map_err(|error| super::super::error::detailed(Status::unavailable("cannot resolve primary run receipt; retry lookup"), &error))?
        .ok_or_else(|| Status::not_found("run request not found"))?;
    Ok(receipt)
}

fn command_id(id: &str) -> Result<CommandId, Status> {
    value::text(id)?;
    CommandId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid command UUID"))
}

fn action(action: dokime::domain::model::execution::wire::ExecutionCommandAction, step_id: Option<&str>) -> Result<StepCommand, Status> {
    match (action, step_id) {
        (dokime::domain::model::execution::wire::ExecutionCommandAction::Next, None) => Ok(StepCommand::Next),
        (dokime::domain::model::execution::wire::ExecutionCommandAction::Finish, None) => Ok(StepCommand::Finish),
        (dokime::domain::model::execution::wire::ExecutionCommandAction::RunStep, Some(id)) => {
            value::text(id)?;
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
    if let Some(detail) = &receipt.detail {
        value::text(detail)?;
    }
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

/* === Tests === */

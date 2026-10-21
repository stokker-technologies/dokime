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

// Local Imports
use super::*;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::domain::model::identity::wire as common;
use dokime::domain::operations::execution as wire;
use dokime::runtime::command::{CommandReceipt, CommandStatus};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Execution {
    runtime: Arc<RuntimeHandle>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum Operations {
    Execution(Execution),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Execution {
    type Declaration = wire::StartExecution;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(runtime: Arc<RuntimeHandle>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }
    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut wire::StartExecution, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::execution::wire::RunReply>, Status> {
        let context = super::control::command_context(&context, None)?;
        value::text(&request.execution_id)?;
        let id = dokime::domain::model::identity::TestExecutionId::from_uuid(request.execution_id.clone())
            .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        let mode = match request.mode {
            dokime::domain::model::execution::wire::ExecutionMode::Automatic => dokime::domain::model::execution::ExecutionMode::Automatic,
            dokime::domain::model::execution::wire::ExecutionMode::Interactive => {
                dokime::domain::model::execution::ExecutionMode::Interactive
            }
        };
        let result = self.runtime.start_execution(id, mode, context).await;
        receipt(result, self.runtime.process_epoch().to_string()).map(Into::into)
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

/* === Private Functions === */

fn receipt(receipt: CommandReceipt, epoch: String) -> Result<dokime::domain::model::execution::wire::RunReply, Status> {
    let outcome = match receipt.status {
        CommandStatus::Rejected => {
            let detail = receipt.rejection.ok_or_else(|| Status::internal("rejected receipt omitted reason"))?;
            dokime::domain::model::execution::wire::RunReplyOutcome::Rejected(super::rejection(
                detail.code,
                detail.message,
                detail.validation_issues,
            ))
        }
        CommandStatus::Accepted => {
            let at = time::OffsetDateTime::parse(receipt.accepted_at.as_str(), &time::format_description::well_known::Rfc3339)
                .map_err(|_| Status::internal("invalid admission timestamp"))?;
            dokime::domain::model::execution::wire::RunReplyOutcome::Accepted(dokime::domain::model::execution::wire::Admission {
                command_id: receipt.command_id.to_string(),
                job_id: receipt.job_id.map(|id| id.to_string()),
                cursor: Some(Cursor { epoch, stream: crate::grpc::observation::STREAM.into(), position: receipt.runtime_revision }),
                accepted_at: Some(common::Timestamp { seconds: at.unix_timestamp(), nanos: at.nanosecond() as i32 }),
            })
        }
    };
    Ok(dokime::domain::model::execution::wire::RunReply { outcome: Some(outcome) })
}

/* === Tests === */

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

use super::*;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::domain::model::identity::wire as common;
use dokime::operations::execution as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::command::{CommandReceipt, CommandStatus};

struct Resources {
    runtime: RuntimeHandle,
}

impl Serve<wire::StartExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::StartExecution, ctx: Context<'a, wire::StartExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::RunReply> {
        Box::pin(async move {
            let context = super::control::command_context(ctx.call(), None)?;
            let id = dokime::domain::model::identity::TestExecutionId::from_uuid(request.execution_id.clone())
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
            let mode = match request.mode {
                dokime::domain::model::execution::wire::ExecutionMode::Automatic => {
                    dokime::domain::model::execution::ExecutionMode::Automatic
                }
                dokime::domain::model::execution::wire::ExecutionMode::Interactive => {
                    dokime::domain::model::execution::ExecutionMode::Interactive
                }
            };
            let result = self.runtime.start_execution(id, mode, context).await;
            receipt(result, self.runtime.process_epoch().to_string()).map(Into::into)
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone() });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::StartExecution>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::StartExecution>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

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
    Ok(dokime::domain::model::execution::wire::RunReply { outcome: Some(outcome) })
}

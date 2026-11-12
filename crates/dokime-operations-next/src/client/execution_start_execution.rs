/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Start Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.start_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionStartExecutionRequest, v1::StartExecutionReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionStartExecutionRequest) -> Result<v1::StartExecutionReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let controls = context.host.controls(&request.execution_id)?;
        let mode = match v1::ExecutionMode::try_from(request.mode) {
            Ok(v1::ExecutionMode::Interactive) => v1::ExecutionMode::Interactive,
            Ok(v1::ExecutionMode::Automatic | v1::ExecutionMode::Unspecified) => v1::ExecutionMode::Automatic,
            _ => return Err(Error::Invalid("Start Execution: Choose Automatic or Interactive Mode".into())),
        };
        if controls.can_cancel {
            if controls.mode != mode as i32 {
                return Err(Error::Conflict("Start Execution: Resolve the Existing Run in Its Original Mode".into()));
            }
        } else {
            context.host.start_existing(&request.execution_id, mode).await?;
        }
        Ok(v1::StartExecutionReply {
            outcome: Some(v1::StartExecutionOutcome {
                kind: Some(v1::start_execution_outcome::Kind::Accepted(v1::Admission {
                    command_id: request.execution_id,
                    cursor: Some(super::execution_state::observed(context)),
                    accepted_at: Some(super::projection::timestamp(context.host.now_ns())),
                    ..Default::default()
                })),
            }),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

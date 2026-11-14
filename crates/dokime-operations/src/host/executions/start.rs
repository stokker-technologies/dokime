/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Start

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::ExecutionsStartRequest;
use dokime_protocol::host::ExecutionsStartResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Start";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsStartRequest, ExecutionsStartResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsStartRequest) -> Result<ExecutionsStartResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let controls = context.host.controls(&request.name)?;
        let mode = match ExecutionMode::try_from(request.mode) {
            Ok(ExecutionMode::Interactive) => ExecutionMode::Interactive,
            Ok(ExecutionMode::Automatic | ExecutionMode::Unspecified) => ExecutionMode::Automatic,
            _ => return Err(Error::Invalid("Start Execution: Choose an Execution Mode".into())),
        };
        if controls.can_cancel {
            if controls.mode != mode as i32 {
                return Err(Error::Conflict("Start Execution: Resolve the Existing Run Mode".into()));
            }
        } else {
            context.host.start_existing(&request.name, mode).await?;
        }
        Ok(ExecutionsStartResponse {
            execution: Some(crate::host::execution_state::detail(context, &request.name)?),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Start";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsStartRequest, dokime_protocol::host::ExecutionsStartResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsStartRequest,
) -> Result<dokime_protocol::host::ExecutionsStartResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let controls = context.host.controls(&request.name)?;
        let mode = match dokime_protocol::host::ExecutionMode::try_from(request.mode) {
            Ok(dokime_protocol::host::ExecutionMode::Interactive) => dokime_protocol::host::ExecutionMode::Interactive,
            Ok(dokime_protocol::host::ExecutionMode::Automatic | dokime_protocol::host::ExecutionMode::Unspecified) => {
                dokime_protocol::host::ExecutionMode::Automatic
            }
            _ => return Err(Error::Invalid("Start Execution: Choose an Execution Mode".into())),
        };
        if controls.can_cancel {
            if controls.mode != mode as i32 {
                return Err(Error::Conflict("Start Execution: Resolve the Existing Run Mode".into()));
            }
        } else {
            context.host.start_existing(&request.name, mode).await?;
        }
        Ok(dokime_protocol::host::ExecutionsStartResponse {
            execution: Some(crate::host::execution_state::detail(context, &request.name)?),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

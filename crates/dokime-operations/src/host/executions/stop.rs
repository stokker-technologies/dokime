/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Stop

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::RunAction;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionsStopRequest;
use dokime_protocol::host::ExecutionsStopResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Stop";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsStopRequest) -> Result<ExecutionsStopResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        context.host.control_execution(&request.name, RunAction::Stop)?;
        Ok(ExecutionsStopResponse {
            execution: Some(crate::host::execution_state::detail(context, &request.name)?),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsStopRequest, ExecutionsStopResponse, context, "dokime.host.v1", "Executions", "Stop");

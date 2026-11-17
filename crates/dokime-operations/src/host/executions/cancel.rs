/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Cancel

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::RunAction;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionsCancelRequest;
use dokime_protocol::host::ExecutionsCancelResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Cancel";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsCancelRequest) -> Result<ExecutionsCancelResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        context.host.control_execution(&request.name, RunAction::Cancel)?;
        Ok(ExecutionsCancelResponse {
            execution: Some(crate::host::execution_state::detail(context, &request.name)?),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsCancelRequest, ExecutionsCancelResponse, context, "dokime.host.v1", "Executions", "Cancel");

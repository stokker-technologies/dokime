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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Cancel";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsCancelRequest, dokime_protocol::host::ExecutionsCancelResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsCancelRequest,
) -> Result<dokime_protocol::host::ExecutionsCancelResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        context.host.control_execution(&request.name, crate::host::RunAction::Cancel)?;
        Ok(dokime_protocol::host::ExecutionsCancelResponse {
            execution: Some(crate::host::execution_state::detail(context, &request.name)?),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

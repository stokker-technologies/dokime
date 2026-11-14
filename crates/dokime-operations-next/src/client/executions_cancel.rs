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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Executions/Cancel";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::ExecutionsCancelRequest, dokime_protocol::client::ExecutionsCancelResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::ExecutionsCancelRequest,
) -> Result<dokime_protocol::client::ExecutionsCancelResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        context.host.control_execution(&request.name, super::RunAction::Cancel)?;
        Ok(dokime_protocol::client::ExecutionsCancelResponse {
            execution: Some(super::execution_state::detail(context, &request.name)?),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

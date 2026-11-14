/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsDeleteResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsDeleteRequest, ExecutionsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsDeleteRequest) -> Result<ExecutionsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if let Some(reply) = context.host.store().deleted_execution(&request.request_id, &request.name)? {
            return Ok(reply);
        }
        let execution = crate::host::execution_state::detail(context, &request.name)?;
        let cut = context.host.store().cut(&request.name)?;
        let response = ExecutionsDeleteResponse {
            execution: Some(execution),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        };
        context.host.store().delete_sealed_execution(&request.request_id, &request.name, &cut.seal, &response)?;
        context.host.signal_change();
        Ok(response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsDeleteRequest, ExecutionsDeleteResponse, context, "dokime.host.v1", "Executions", "Delete");

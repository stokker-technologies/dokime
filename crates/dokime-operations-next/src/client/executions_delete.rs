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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Executions/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsDeleteRequest, v1::ExecutionsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsDeleteRequest) -> Result<v1::ExecutionsDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if let Some(reply) = context.host.store().deleted_execution(&request.request_id, &request.name)? {
            return Ok(reply);
        }
        let execution = super::execution_state::detail(context, &request.name)?;
        let cut = context.host.store().cut(&request.name)?;
        let response = v1::ExecutionsDeleteResponse {
            execution: Some(execution),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        };
        context.host.store().delete_sealed_execution(&request.request_id, &request.name, &cut.seal, &response)?;
        context.host.signal_change();
        Ok(response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CasesDeleteRequest, dokime_protocol::host::CasesDeleteResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CasesDeleteRequest,
) -> Result<dokime_protocol::host::CasesDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        crate::host::execution_state::cycle(context, &request.parent)?;
        context.host.store().delete_planned_case(&request.parent, &request.name)?;
        context.host.signal_change();
        Ok(dokime_protocol::host::CasesDeleteResponse {
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

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

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::CasesDeleteRequest;
use dokime_protocol::host::CasesDeleteResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesDeleteRequest, CasesDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: CasesDeleteRequest) -> Result<CasesDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        crate::host::execution_state::cycle(context, &request.parent)?;
        context.host.store().delete_planned_case(&request.parent, &request.name)?;
        context.host.signal_change();
        Ok(CasesDeleteResponse {
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CasesDeleteRequest, CasesDeleteResponse, context, "dokime.host.v1", "Cases", "Delete");

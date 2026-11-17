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
use crate::{Result, host::Context};
use dokime_protocol::host::CasesDeleteRequest;
use dokime_protocol::host::CasesDeleteResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: CasesDeleteRequest) -> Result<CasesDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        crate::host::execution_state::cycle(context, &request.parent).await?;
        let receipt = super::storage::delete(context, &request.parent, &request.name, &request.request_id).await?;
        context.host.signal_change();
        Ok(CasesDeleteResponse { receipt: Some(receipt) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CasesDeleteRequest, CasesDeleteResponse, context, "dokime.host.v1", "Cases", "Delete");

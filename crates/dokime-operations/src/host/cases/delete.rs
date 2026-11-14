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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Cases/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::CasesDeleteRequest, dokime_protocol::client::CasesDeleteResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::CasesDeleteRequest,
) -> Result<dokime_protocol::client::CasesDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::execution_state::cycle(context, &request.parent)?;
        context.host.store().delete_planned_case(&request.parent, &request.name)?;
        context.host.signal_change();
        Ok(dokime_protocol::client::CasesDeleteResponse {
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

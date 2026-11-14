/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CasesListRequest, dokime_protocol::host::CasesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CasesListRequest,
) -> Result<dokime_protocol::host::CasesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("{ID}/{}/{}", request.parent, String::new()), request.at.as_ref(), async {
                let cases = crate::host::execution_state::cases(context, &request.parent)?;
                Ok(dokime_protocol::host::CasesListResponse { cases, ..Default::default() })
            })
            .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

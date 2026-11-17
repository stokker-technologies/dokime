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
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::CasesListRequest;
use dokime_protocol::host::CasesListResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesListRequest, CasesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CasesListRequest) -> Result<CasesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let owner = if request.parent.is_empty() {
            Scope::Root(&context.host.identity().backend_id).identifier()?
        } else {
            Scope::Cycle(&request.parent).identifier()?
        };
        let (mut reply, cut) = crate::host::views::read(context, owner, &format!("{ID}/{}/", request.parent), request.at.as_ref(), async {
            let cases = crate::host::execution_state::cases(context, &request.parent).await?;
            Ok(CasesListResponse { cases, ..Default::default() })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CasesListRequest, CasesListResponse, context, "dokime.host.v1", "Cases", "List");

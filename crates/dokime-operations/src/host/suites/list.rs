/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Suites List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::SuitesListRequest;
use dokime_protocol::host::SuitesListResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Suites/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SuitesListRequest, SuitesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SuitesListRequest) -> Result<SuitesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/suites/{}", request.parent);
        let owner = Scope::Root(&context.host.identity().backend_id).identifier()?;
        let (saved, cut) = crate::host::views::read(context, owner, &key, at.as_ref(), async {
            let inventory = crate::host::catalog_inventory::Inventory::read(context).await?;
            let values = inventory.suites.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
            Ok(SuitesListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(SuitesListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SuitesListRequest, SuitesListResponse, context, "dokime.host.v1", "Suites", "List");

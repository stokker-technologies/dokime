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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Suites/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SuitesListRequest, dokime_protocol::host::SuitesListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SuitesListRequest,
) -> Result<dokime_protocol::host::SuitesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/suites/{}", request.parent);
        let (saved, cut) = crate::host::views::read(context, &key, at.as_ref(), async {
            let inventory = crate::host::catalog_inventory::Inventory::read(context).await?;
            let values = inventory.suites.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
            Ok(dokime_protocol::host::SuitesListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(dokime_protocol::host::SuitesListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

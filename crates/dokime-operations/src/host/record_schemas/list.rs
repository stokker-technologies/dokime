/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Record Schemas List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.RecordSchemas/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::RecordSchemasListRequest, dokime_protocol::host::RecordSchemasListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::RecordSchemasListRequest,
) -> Result<dokime_protocol::host::RecordSchemasListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/record_schemas/{}", request.parent);
        let (saved, cut) = crate::host::views::read(context, &key, at.as_ref(), async {
            let inventory = if request.parent.is_empty() {
                crate::host::catalog_inventory::Inventory::read(context).await?
            } else {
                crate::host::catalog_inventory::Inventory::captured(context, &request.parent)?
            };
            let values = inventory.records;
            Ok(dokime_protocol::host::RecordSchemasListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(dokime_protocol::host::RecordSchemasListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

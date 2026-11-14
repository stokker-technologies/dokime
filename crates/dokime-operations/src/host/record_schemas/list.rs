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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::RecordSchemasListRequest;
use dokime_protocol::host::RecordSchemasListResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.RecordSchemas/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<RecordSchemasListRequest, RecordSchemasListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: RecordSchemasListRequest) -> Result<RecordSchemasListResponse> {
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
            Ok(RecordSchemasListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(RecordSchemasListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

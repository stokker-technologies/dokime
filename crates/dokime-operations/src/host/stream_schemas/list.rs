/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Stream Schemas List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::StreamSchemasListRequest;
use dokime_protocol::host::StreamSchemasListResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.StreamSchemas/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<StreamSchemasListRequest, StreamSchemasListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: StreamSchemasListRequest) -> Result<StreamSchemasListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/stream_schemas/{}", request.parent);
        let (saved, cut) = crate::host::views::read(context, &key, at.as_ref(), async {
            let inventory = if request.parent.is_empty() {
                crate::host::catalog_inventory::Inventory::read(context).await?
            } else {
                crate::host::catalog_inventory::Inventory::captured(context, &request.parent)?
            };
            let values = inventory.streams;
            Ok(StreamSchemasListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(StreamSchemasListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

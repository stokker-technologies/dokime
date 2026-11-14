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

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.StreamSchemas/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::StreamSchemasListRequest, dokime_protocol::client::StreamSchemasListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::StreamSchemasListRequest,
) -> Result<dokime_protocol::client::StreamSchemasListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let at = request.at.clone();
        let key = format!("catalog/stream_schemas/{}", request.parent);
        let (saved, cut) = super::views::read(context, &key, at.as_ref(), async {
            let inventory = if request.parent.is_empty() {
                super::catalog_inventory::Inventory::read(context).await?
            } else {
                super::catalog_inventory::Inventory::captured(context, &request.parent)?
            };
            let values = inventory.streams;
            Ok(dokime_protocol::client::StreamSchemasListResponse { values, ..Default::default() })
        })
        .await?;
        Ok(dokime_protocol::client::StreamSchemasListResponse { resolved_at: Some(cut), ..saved })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

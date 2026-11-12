/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.catalog.list_telemetry_schemas";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogListTelemetrySchemasRequest, v1::ListTelemetrySchemasReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogListTelemetrySchemasRequest) -> Result<v1::ListTelemetrySchemasReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context).await?;
        inventory.check(request.cursor.as_ref())?;
        let after = &request.after_stream;
        if !after.is_empty() && !inventory.streams.iter().any(|item| &item.stream == after) {
            return Err(Error::Invalid("Continue Catalog: Select an Existing After Identifier".into()));
        }
        let limit = super::page_size(request.limit, context.settings.page_items)?;
        let mut entries = inventory.streams.into_iter().filter(|item| item.stream > *after);
        let schemas: Vec<_> = entries
            .by_ref()
            .take(limit)
            .map(|item| v1::TelemetrySchemaIdentity {
                stream: item.stream,
                schema_version: item.schema_version,
                metadata: item.metadata,
            })
            .collect();
        let next = entries.next().and_then(|_| schemas.last().map(|item| item.stream.clone()));
        Ok(v1::ListTelemetrySchemasReply {
            schemas,
            next_after_stream: next,
            cursor: Some(inventory.cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

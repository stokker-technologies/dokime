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

pub const ID: &str = "dokime.catalog.list_record_schemas";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogListRecordSchemasRequest, v1::ListRecordSchemasReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogListRecordSchemasRequest) -> Result<v1::ListRecordSchemasReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context)?;
        inventory.check(request.cursor.as_ref())?;
        let after = &request.after_record_type;
        if !after.is_empty() && !inventory.records.iter().any(|item| &item.record_type == after) { return Err(Error::Invalid("Continue Catalog: Select an Existing After Identifier".into())); }
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let mut entries = inventory.records.into_iter().filter(|item| item.record_type > *after);
        let schemas: Vec<_> = entries.by_ref().take(limit).map(|item| v1::RecordSchemaIdentity { record_type: item.record_type, schema_version: item.schema_version, metadata: item.metadata }).collect();
        let next = entries.next().and_then(|_| schemas.last().map(|item| item.record_type.clone()));
        Ok(v1::ListRecordSchemasReply { schemas, next_after_record_type: next, cursor: Some(inventory.cursor) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

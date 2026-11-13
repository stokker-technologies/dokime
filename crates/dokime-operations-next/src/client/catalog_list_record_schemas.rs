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
use crate::{Result, client::Context};

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
pub async fn handle(context: &Context, _request: v1::CatalogListRecordSchemasRequest) -> Result<v1::ListRecordSchemasReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context).await?;
        let schemas = inventory
            .records
            .into_iter()
            .map(|item| v1::RecordSchemaIdentity {
                record_type: item.record_type,
                schema_version: item.schema_version,
                metadata: item.metadata,
            })
            .collect();
        Ok(v1::ListRecordSchemasReply { schemas })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

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
pub async fn handle(context: &Context, _request: v1::CatalogListTelemetrySchemasRequest) -> Result<v1::ListTelemetrySchemasReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context).await?;
        let schemas = inventory
            .streams
            .into_iter()
            .map(|item| v1::TelemetrySchemaIdentity {
                stream: item.stream,
                schema_version: item.schema_version,
                metadata: item.metadata,
            })
            .collect();
        Ok(v1::ListTelemetrySchemasReply { schemas })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

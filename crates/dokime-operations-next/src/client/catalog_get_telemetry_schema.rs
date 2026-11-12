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

pub const ID: &str = "dokime.catalog.get_telemetry_schema";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogGetTelemetrySchemaRequest, v1::TelemetrySchema>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogGetTelemetrySchemaRequest) -> Result<v1::TelemetrySchema> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context)?;
        inventory.streams.into_iter().find(|item| item.stream == request.stream).ok_or_else(|| Error::Invalid("Read Schema: Admit the Plugin Declaring This Identifier".into()))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

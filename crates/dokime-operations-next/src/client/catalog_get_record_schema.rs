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

pub const ID: &str = "dokime.catalog.get_record_schema";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CatalogGetRecordSchemaRequest, v1::RecordSchema>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogGetRecordSchemaRequest) -> Result<v1::RecordSchema> {
    super::complete(ID, context.settings.read_timeout, async {
        let inventory = super::catalog_inventory::Inventory::read(context)?;
        inventory.records.into_iter().find(|item| item.record_type == request.record_type).ok_or_else(|| Error::Invalid("Read Schema: Admit the Plugin Declaring This Identifier".into()))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

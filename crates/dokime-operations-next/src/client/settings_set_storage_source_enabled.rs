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

pub const ID: &str = "dokime.settings.set_storage_source_enabled";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SettingsSetStorageSourceEnabledRequest, v1::StorageSources>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SettingsSetStorageSourceEnabledRequest) -> Result<v1::StorageSources> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.identifier != "native" || !request.enabled { return Err(Error::Invalid("Configure Storage: Keep the Native Writable Source Enabled".into())); }
        Ok(v1::StorageSources { sources: vec![v1::StorageSource { identifier: "native".into(), adapter: "sqlite-arrow".into(), priority: 0, enabled: true }] })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

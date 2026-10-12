/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */
pub use dokime::storage::sqlite::Source as StorageSource;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn storage_sources(&self) -> Result<Vec<StorageSource>> {
        Ok(self.local_storage.sources().await?)
    }

    pub async fn reorder_storage_sources(&self, identifiers: Vec<String>) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        if identifiers.is_empty()
            || identifiers.len() > 1024
            || identifiers
                .iter()
                .any(|identifier| identifier.is_empty() || identifier.len() > 256)
        {
            return Err(Error::validation(
                "invalid_storage_sources",
                "provide 1 to 1024 source identifiers of 1 to 256 bytes",
                Vec::new(),
            ));
        }
        self.local_storage
            .reorder_sources(identifiers)
            .await
            .map_err(settings_error)
    }

    pub async fn set_storage_source_enabled(&self, identifier: &str, enabled: bool) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        if identifier.is_empty() || identifier.len() > 256 {
            return Err(Error::validation(
                "invalid_storage_source",
                "provide a source identifier of 1 to 256 bytes",
                Vec::new(),
            ));
        }
        self.local_storage
            .set_source_enabled(identifier, enabled)
            .await
            .map_err(settings_error)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn settings_error(error: dokime::storage::sqlite::Error) -> Error {
    match error {
        dokime::storage::sqlite::Error::Catalog(message) => {
            Error::validation("invalid_storage_sources", message, Vec::new())
        }
        error => error.into(),
    }
}

/* === Tests === */

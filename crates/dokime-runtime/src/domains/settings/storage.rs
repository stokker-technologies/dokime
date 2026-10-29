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

pub use crate::storage::sqlite::Source as StorageSource;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{RuntimeHandle, runtime::Error, runtime::Result};

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
        self.local_storage.reorder_sources(identifiers).await.map_err(settings_error)
    }

    pub async fn set_storage_source_enabled(&self, identifier: &str, enabled: bool) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        self.local_storage.set_source_enabled(identifier, enabled).await.map_err(settings_error)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn settings_error(error: crate::storage::sqlite::Error) -> Error {
    match error {
        crate::storage::sqlite::Error::Catalog(message) => Error::validation("invalid_storage_sources", message, Vec::new()),
        error => error.into(),
    }
}

/* === Tests === */

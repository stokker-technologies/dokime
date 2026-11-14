/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Flush Policy

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, time::Duration};

// Third Party Imports

// Local Imports
use crate::{Error, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug)]
pub struct StorageSettings {
    pub flush_rows: NonZeroUsize,
    pub flush_interval: Duration,
    pub pending_bytes: NonZeroUsize,
    pub query_bytes: NonZeroUsize,
    pub previews: PreviewSettings,
}

#[derive(Clone, Copy, Debug)]
pub struct PreviewSettings {
    pub bucket_count: NonZeroUsize,
    pub bucket_interval: Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn storage_settings(&self) -> Result<StorageSettings> {
        Ok(self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry settings lock failed"))?.settings)
    }
}

/* === Public Functions === */

pub fn storage_defaults() -> StorageSettings {
    StorageSettings {
        flush_rows: NonZeroUsize::new(65_536).unwrap(),
        flush_interval: Duration::from_secs(5),
        pending_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
        query_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
        previews: PreviewSettings {
            bucket_count: NonZeroUsize::new(256).unwrap(),
            bucket_interval: Duration::from_millis(1),
        },
    }
}

/* === Private Functions === */

/* === Tests === */

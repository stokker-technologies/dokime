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
use crate::Store;

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
    /// The Most Fine Value Bins a Field's Histogram Keeps; a Plot's Bins Rebin Them Within One Fine Width.
    pub histogram_bins: NonZeroUsize,
    /// The Least Time From One Live Plot Page's Start to the Next; Changes in Between Coalesce Into One Page.
    pub refresh_interval: Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn storage_settings(&self) -> StorageSettings {
        self.settings
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
            histogram_bins: NonZeroUsize::new(4096).unwrap(),
            refresh_interval: Duration::from_millis(200),
        },
    }
}

/* === Private Functions === */

/* === Tests === */

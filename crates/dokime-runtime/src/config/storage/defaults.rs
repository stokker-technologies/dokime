/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Dokime Storage Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::{StorageFilesystemSettings, StorageRequestSettings, StorageResponseSettings, StorageSettings};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn storage_defaults() -> StorageSettings {
    let active = std::thread::available_parallelism().map_or(4, NonZeroUsize::get).clamp(4, 16);
    StorageSettings {
        filesystem: StorageFilesystemSettings {
            poll_interval: std::time::Duration::from_millis(100),
        },
        request: StorageRequestSettings {
            max_pending_queries: nonzero(128),
            max_active_queries: nonzero(active),
            max_active_file_operations: nonzero(active),
            max_pending_write_batches: nonzero(256),
            max_pending_write_bytes: nonzero(64 * 1024 * 1024),
        },
        response: StorageResponseSettings {
            batch_rows: nonzero(65_536),
            batch_bytes: nonzero(8 * 1024 * 1024),
        },
    }
}

/* === Private Functions === */

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value.max(1)).unwrap()
}

/* === Tests === */

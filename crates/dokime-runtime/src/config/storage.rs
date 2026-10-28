/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod defaults;

/* === Re-Exports === */

pub use defaults::storage_defaults;

/* === Imports === */

// Standard Library Imports
use std::num::{NonZeroU64, NonZeroUsize};

// Third Party Imports
use harmos::config::{ManagedStorageSettings, StorageStateSettings};
use harmos::storage::{ReadLimits, SchemaKey};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn managed_storage_defaults() -> ManagedStorageSettings {
    let limits = ReadLimits {
        rows: NonZeroUsize::new(256).unwrap(),
        bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
    };
    ManagedStorageSettings {
        replay: limits,
        capture: limits,
        snapshots: Some(harmos::state::SnapshotPolicy {
            schema: SchemaKey {
                payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
                id: harmos::Identifier::new("dokime.runtime.state").unwrap(),
                version: 1.try_into().unwrap(),
            },
            every: NonZeroU64::new(1024).unwrap(),
            max_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
        }),
        journal_tail: 64,
    }
}

// One Reader Per Default Active Query; 16,384 Four-KiB WAL Pages (64 MiB) Between Checkpoints.
#[cfg(feature = "storage")]
pub fn local_storage_defaults() -> crate::storage::sqlite::Settings {
    crate::storage::sqlite::Settings {
        readers: storage_defaults().request.max_active_queries,
        checkpoint_pages: std::num::NonZeroU32::new(16_384).unwrap(),
    }
}

pub fn storage_state_defaults() -> StorageStateSettings {
    StorageStateSettings {
        max_entry_bytes: NonZeroUsize::new(731_820).unwrap(),
        max_checkpoint_bytes: NonZeroUsize::new(1_463_640).unwrap(),
    }
}

/* === Private Functions === */

/* === Tests === */

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
use harmos::storage::{
    StorageBudget, StorageRequestSettings, StorageResponseSettings, StorageSettings,
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Dokime's storage policy: a quarter of memory for reads, a twentieth for
/// pending writes, active queries by core count, and response batches of at
/// most an eighth of the read budget.
pub fn storage_defaults() -> StorageSettings {
    let memory = effective_memory();
    let read = nonzero(memory / 4);
    let active = std::thread::available_parallelism()
        .map_or(4, NonZeroUsize::get)
        .clamp(4, 16);
    StorageSettings {
        budget: StorageBudget {
            read_memory_bytes: read,
        },
        request: StorageRequestSettings {
            max_pending_queries: nonzero(128),
            max_active_queries: nonzero(active),
            max_pending_write_batches: nonzero(256),
            max_pending_write_bytes: nonzero(memory / 20),
        },
        response: StorageResponseSettings {
            batch_rows: nonzero(65_536),
            // One response batch takes at most an eighth of the shared read
            // budget, so a single page never credits the whole pool.
            batch_bytes: nonzero(8 * 1024 * 1024).min(nonzero(read.get() / 8)),
        },
    }
}

/* === Private Functions === */
fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value.max(1)).unwrap()
}

fn effective_memory() -> usize {
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
    );
    let physical = match system.total_memory() {
        0 => 4 * 1024 * 1024 * 1024,
        bytes => bytes,
    };
    let effective = system
        .cgroup_limits()
        .map(|limits| limits.total_memory)
        .filter(|bytes| *bytes > 0)
        .map_or(physical, |bytes| physical.min(bytes));
    usize::try_from(effective).unwrap_or(usize::MAX)
}

/* === Tests === */

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Each test binary includes this file and uses its own subset.
#![allow(dead_code)]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
use std::path::PathBuf;
// Third Party Imports

// Local Imports
use dokime::core::Metadata;
use dokime_app::RuntimeConfig;
use dokime_app::config::StorageReadSettings;

/* === Constants === */
/// One story's read ledger. A story on a loaded machine keeps more live views
/// open at once than an interactive session does (follows, captures and
/// finalization overlap while the cycle is still settling), and point reads
/// fail rather than wait while views hold the ledger. Reservations are
/// accounting, not allocations, so sixteen 32 MiB views cost nothing until used.
pub const STORY_READ_MEMORY: NonZeroUsize = NonZeroUsize::new(512 * 1024 * 1024).unwrap();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// A runtime configuration whose read budgets fit one story on a loaded machine.
pub fn runtime_config(root: impl Into<PathBuf>, hub_metadata: Metadata) -> RuntimeConfig {
    let mut config = RuntimeConfig::new(root, hub_metadata);
    story_reads(&mut config.storage_reads);
    config
}

/// The read settings every story runtime gets; views a story opens directly
/// must carry the same execution limits as the runtime they read.
pub fn story_settings() -> StorageReadSettings {
    let mut reads = StorageReadSettings::default();
    story_reads(&mut reads);
    reads
}

/// Sizes reads for one story: a ledger for its overlapping views, and the
/// admissible ceiling as its time budget so no read outcome depends on how
/// loaded the machine is. Stories that assert a deadline set their own after.
pub fn story_reads(reads: &mut StorageReadSettings) {
    reads.memory_bytes = STORY_READ_MEMORY;
    reads.execution.timeout = reads.execution_ceiling.timeout;
}

/// The shipped ledger for a story that proves a transfer or a report stays
/// inside the shipped budgets while moving more bytes than they hold, keeping
/// the load-independent time budget. Such a story really allocates what it
/// reserves, so nextest runs it in the `whole-read-budget` group, one at a time.
pub fn whole_read_budget(reads: &mut StorageReadSettings) {
    reads.memory_bytes = StorageReadSettings::default().memory_bytes;
}

/* === Private Functions === */

/* === Tests === */

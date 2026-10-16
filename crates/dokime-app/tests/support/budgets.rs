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
use dokime_app::config::StorageSettings;
use harmos::storage::StorageProductionPreparation;

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
    story_reads(&mut config.storage);
    config
}

/// The read settings every story runtime gets; views a story opens directly
/// must carry the same execution limits as the runtime they read.
pub fn story_settings() -> StorageSettings {
    let mut reads = StorageSettings::default();
    story_reads(&mut reads);
    reads
}

/// Sizes reads for one story: a ledger for its overlapping views, and the
/// admissible ceiling as its time budget so no read outcome depends on how
/// loaded the machine is. Stories that assert a deadline set their own after.
pub fn story_reads(reads: &mut StorageSettings) {
    reads.budget.read_memory_bytes = STORY_READ_MEMORY;
    reads.interactive.memory_bytes = reads.interactive.memory_bytes.min(STORY_READ_MEMORY);
    reads.ceiling.memory_bytes = reads.ceiling.memory_bytes.min(STORY_READ_MEMORY);
    reads.decoded_bytes = reads.decoded_bytes.min(STORY_READ_MEMORY);
    reads.page_bytes_ceiling = reads.page_bytes_ceiling.min(STORY_READ_MEMORY);
    let scan = NonZeroUsize::new(
        (STORY_READ_MEMORY.get() / reads.request.max_active_queries.get() / 32).max(1),
    )
    .unwrap();
    reads.page_bytes = reads.page_bytes.min(scan);
    reads.interactive.timeout = reads.ceiling.timeout;
}

/// The shipped ledger for a story that proves a transfer or a report stays
/// inside the shipped budgets while moving more bytes than they hold, keeping
/// the load-independent time budget. Such a story really allocates what it
/// reserves, so nextest runs it in the `whole-read-budget` group, one at a time.
pub fn whole_read_budget(reads: &mut StorageSettings) {
    reads.budget.read_memory_bytes = StorageSettings::default().budget.read_memory_bytes;
}

/// Holds the storage read ledger until only `headroom` bytes remain free,
/// whatever the ledger's size, so a story can make the next reader wait for
/// capacity. The ledger caps each reservation at its whole size, so this claims
/// it in small chunks; dropping the returned claims releases it.
pub fn hold_all_but(
    storage: &dokime::storage::Manager,
    headroom: NonZeroUsize,
) -> Vec<StorageProductionPreparation> {
    const CHUNK: usize = 1024 * 1024;
    let mut held = claim_until_exhausted(storage, CHUNK);
    held.extend(claim_until_exhausted(storage, 256));
    let released = headroom.get().div_ceil(CHUNK);
    assert!(
        released <= held.len(),
        "ledger cannot leave {headroom} bytes free"
    );
    held.drain(..released);
    held
}

/* === Private Functions === */
/// Claims `bytes` at a time until the ledger refuses for lack of capacity.
fn claim_until_exhausted(
    storage: &dokime::storage::Manager,
    bytes: usize,
) -> Vec<StorageProductionPreparation> {
    let bytes = NonZeroUsize::new(bytes).unwrap();
    let mut held = Vec::new();
    loop {
        match storage.prepare_production(bytes) {
            Ok(claim) => held.push(claim),
            Err(harmos::Error::ResourceExhausted { .. }) => return held,
            Err(error) => panic!("holding read capacity failed: {error}"),
        }
    }
}

/* === Tests === */

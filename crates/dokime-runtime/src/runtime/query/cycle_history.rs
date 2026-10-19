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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::MemorySize;
use harmos::storage::{ObjectRead, StorageCursor, query::ObjectVersions};

// Local Imports
use super::cycles::{CycleFilter, matches_cycle};
use crate::storage::objects::cycle::Cycle;
use crate::{
    RuntimeHandle,
    runtime::{Error, Result},
};
use dokime_domain::model::execution::result::{CyclePhase, TestCycle};
use dokime_domain::model::identity::{Metadata, TestCycleId};
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CycleBrowseQuery {
    pub at: Option<StorageCursor>,
    pub after: u64,
    pub limit: usize,
    pub max_bytes: usize,
    pub filter: CycleFilter,
}

pub struct CycleBrowsePage {
    pub entries: Vec<SavedCycleSummary>,
    pub at: StorageCursor,
    pub next: Option<u64>,
}

pub struct SavedCycleSummary {
    pub cycle_id: TestCycleId,
    pub metadata: Metadata,
    pub phase: CyclePhase,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub execution_count: usize,
    pub archived: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    /// Read a bounded page from one saved image without retaining the historical catalog.
    /// The current typed storage interface repeats prefix scans on continuation.
    pub async fn browse_cycles(&self, query: CycleBrowseQuery) -> Result<CycleBrowsePage> {
        let saved = self.storage().cursor_saved();
        let missing_snapshot = query.after != 0 && query.at.is_none();
        let at = query.at.unwrap_or(saved.clone());
        let invalid_snapshot = at.history != saved.history || at.sequence > saved.sequence;
        let invalid_page = !(1..=1000).contains(&query.limit) || query.after > at.sequence;
        if missing_snapshot || invalid_snapshot || invalid_page {
            return Err(Error::Operation("invalid historical cycle page limit or cursor".into()));
        }

        let archived = self.state_read().archived_cycles.clone();
        let mut reader =
            self.storage().read_objects::<Cycle>(ObjectRead { at: at.clone(), key: None, versions: ObjectVersions::Latest }).await?;
        let mut entries = Vec::with_capacity(query.limit);
        let mut bytes = std::mem::size_of::<CycleBrowsePage>()
            .saturating_add(entries.capacity().saturating_mul(std::mem::size_of::<SavedCycleSummary>()));
        if bytes > query.max_bytes {
            return Err(harmos::Error::ResourceExhausted { resource: "historical cycle page" }.into());
        }
        let mut last = query.after;
        while let Some(rows) = reader.next().await? {
            for row in rows {
                if row.position.sequence <= query.after {
                    continue;
                }
                let cycle: TestCycle = row.value.try_into()?;
                let is_archived = archived.contains(&cycle.test_cycle_id);
                if cycle.phase != CyclePhase::Sealed || cycle.saved.is_none() || !matches_cycle(&cycle, is_archived, &query.filter) {
                    continue;
                }
                if entries.len() == query.limit {
                    return Ok(CycleBrowsePage { entries, at, next: Some(last) });
                }
                let header_bytes = cycle
                    .metadata
                    .memory_size()
                    .saturating_add(cycle.test_cycle_id.as_str().len())
                    .saturating_add(cycle.created_at.as_str().len())
                    .saturating_add(cycle.updated_at.as_str().len());
                bytes = bytes.saturating_add(header_bytes);
                if bytes > query.max_bytes {
                    return Err(harmos::Error::ResourceExhausted { resource: "historical cycle page" }.into());
                }
                last = row.position.sequence;
                entries.push(SavedCycleSummary {
                    cycle_id: cycle.test_cycle_id,
                    metadata: cycle.metadata,
                    phase: cycle.phase,
                    created_at: cycle.created_at,
                    updated_at: cycle.updated_at,
                    execution_count: cycle.execution_ids.len(),
                    archived: is_archived,
                });
            }
        }
        Ok(CycleBrowsePage { entries, at, next: None })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

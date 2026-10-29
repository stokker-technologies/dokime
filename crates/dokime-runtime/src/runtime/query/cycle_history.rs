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
    pub limit: Option<usize>,
    pub max_bytes: usize,
    pub filter: CycleFilter,
}

pub struct CycleBrowsePage {
    pub entries: crate::storage::ReadRows<SavedCycleSummary>,
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

impl harmos::MemorySize for SavedCycleSummary {
    fn memory_size(&self) -> usize {
        let fields = [
            self.cycle_id.memory_size().saturating_sub(std::mem::size_of_val(&self.cycle_id)),
            self.metadata.memory_size().saturating_sub(std::mem::size_of_val(&self.metadata)),
            self.created_at.memory_size().saturating_sub(std::mem::size_of_val(&self.created_at)),
            self.updated_at.memory_size().saturating_sub(std::mem::size_of_val(&self.updated_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add)
    }
}

impl RuntimeHandle {
    pub async fn browse_cycles(&self, query: CycleBrowseQuery) -> Result<harmos::storage::ReadBatch<CycleBrowsePage>> {
        let default_limit = 100;
        let limit = query.limit.unwrap_or(default_limit);
        let saved = self.storage().cursor_saved();
        let missing_snapshot = query.after != 0 && query.at.is_none();
        let at = query.at.unwrap_or(saved.clone());
        let invalid_snapshot = at.history != saved.history || at.sequence > saved.sequence;
        let invalid_page = limit == 0 || limit as u64 > 1000 || query.after > at.sequence;
        if missing_snapshot || invalid_snapshot || invalid_page {
            return Err(Error::validation("invalid_history_cursor", "invalid historical cycle page limit or cursor", vec![]));
        }

        let slots = limit
            .checked_mul(std::mem::size_of::<SavedCycleSummary>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<CycleBrowsePage>()))
            .ok_or(harmos::Error::ResourceExhausted { resource: "historical cycle page" })?;
        if slots > query.max_bytes {
            return Err(harmos::Error::ResourceExhausted { resource: "historical cycle page" }.into());
        }
        let archived = self.state_read().archived_cycles.clone();
        let session = self.storage().clone();
        let at = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(at))?;
        let mut reader = session
            .read_objects::<Cycle>(ObjectRead {
                at: (*at).clone(),
                key: None,
                versions: ObjectVersions::Latest,
            })
            .await?;
        let mut entries = harmos::storage::ReadBatch::new(Vec::new());
        let mut count = 0usize;
        let mut bytes = slots;
        let mut last = query.after;
        while let Some(rows) = reader.next().await? {
            let mut more = false;
            let selected = rows.try_map(|rows| -> Result<_> {
                let mut selected = Vec::new();
                selected
                    .try_reserve_exact(rows.len().min(limit - count))
                    .map_err(|_| harmos::Error::ResourceExhausted { resource: "historical cycle page" })?;
                for row in rows {
                    if row.position.sequence <= query.after {
                        continue;
                    }
                    let cycle: TestCycle = row.value.try_into()?;
                    let is_archived = archived.contains(&cycle.test_cycle_id);
                    if cycle.phase != CyclePhase::Sealed || cycle.saved.is_none() || !matches_cycle(&cycle, is_archived, &query.filter) {
                        continue;
                    }
                    if count == limit {
                        more = true;
                        break;
                    }
                    let fields = [
                        cycle.metadata.memory_size(),
                        cycle.test_cycle_id.as_str().len(),
                        cycle.created_at.as_str().len(),
                        cycle.updated_at.as_str().len(),
                    ];
                    for field in fields {
                        bytes = bytes.checked_add(field).ok_or(harmos::Error::ResourceExhausted { resource: "historical cycle page" })?;
                    }
                    if bytes > query.max_bytes {
                        return Err(harmos::Error::ResourceExhausted { resource: "historical cycle page" }.into());
                    }
                    last = row.position.sequence;
                    count += 1;
                    selected.push(SavedCycleSummary {
                        cycle_id: cycle.test_cycle_id,
                        metadata: cycle.metadata,
                        phase: cycle.phase,
                        created_at: cycle.created_at,
                        updated_at: cycle.updated_at,
                        execution_count: cycle.execution_ids.len(),
                        archived: is_archived,
                    });
                }
                Ok(selected)
            })?;
            if !selected.is_empty() {
                entries = crate::storage::projection::push(entries, selected)?;
            }
            if more {
                return at.try_map(|at| {
                    Ok::<_, Error>(CycleBrowsePage {
                        entries: entries.into(),
                        at,
                        next: Some(last),
                    })
                });
            }
        }
        at.try_map(|at| Ok::<_, Error>(CycleBrowsePage { entries: entries.into(), at, next: None }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
use std::{ops::Deref, sync::Arc};

// Third Party Imports
use harmos::storage::{RecordRead, RecordReader, query::RecordSelection};

// Local Imports
use super::{Source, error};
use crate::runtime::Result;
use crate::storage::records::RecordRow;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct History {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    rows: RecordReader<RecordRow>,
    pending: Option<harmos::storage::ReadBatch<Vec<CapturedRow>>>,
    pending_index: usize,
    // Logical Seek: Only Rows Journaled After This Capture Sequence.
    sequence_after: Option<u64>,
    after: u64,
    through: u64,
    done: bool,
}

#[derive(Clone)]
pub struct CapturedRecord {
    page: harmos::storage::ReadBatch<Vec<CapturedRow>>,
    index: usize,
}

struct CapturedRow {
    position: u64,
    current_process: bool,
    record: RecordEntry,
}

/* === Enums === */

pub(super) enum Start {
    Physical(u64),
    Logical(u64),
}

/* === Traits === */

/* === Implementations === */

impl Source {
    pub(crate) async fn saved_filtered_records(
        &self, state_manager: &crate::app::StateManager, through: u64, select: impl Fn(&RecordEntry) -> bool,
    ) -> Result<crate::storage::projection::Pages<RecordEntry>> {
        self.saved_filtered_records_in(&self.storage.clone(), state_manager, through, select).await
    }

    pub(crate) async fn saved_filtered_records_in(
        &self, session: &crate::storage::Manager, state_manager: &crate::app::StateManager, through: u64,
        select: impl Fn(&RecordEntry) -> bool,
    ) -> Result<crate::storage::projection::Pages<RecordEntry>> {
        self.wait_through(state_manager, through).await?;
        let at = self.save_accepted(state_manager).await?;
        self.saved_filtered_records_at_in(session, through, &at, select).await
    }

    pub(crate) async fn saved_filtered_records_at_in(
        &self, session: &crate::storage::Manager, through: u64, at: &harmos::storage::StorageCursor, select: impl Fn(&RecordEntry) -> bool,
    ) -> Result<crate::storage::projection::Pages<RecordEntry>> {
        let mut pages = harmos::storage::ReadBatch::new(Vec::new());
        self.visit_saved_records_at_in(session, through, at, |page| {
            let page = page.try_map(|rows| Ok::<_, crate::runtime::Error>(rows.into_iter().filter(&select).collect()))?;
            let previous = std::mem::replace(&mut pages, harmos::storage::ReadBatch::new(Vec::new()));
            pages = crate::storage::projection::push(previous, page)?;
            Ok(())
        })
        .await?;
        Ok(pages)
    }

    pub(crate) async fn visit_saved_records_at_in(
        &self, session: &crate::storage::Manager, through: u64, at: &harmos::storage::StorageCursor,
        mut visit: impl FnMut(harmos::storage::ReadBatch<Vec<RecordEntry>>) -> Result<()>,
    ) -> Result<()> {
        let mut rows = session
            .read_records::<RecordRow>(RecordRead {
                at: at.clone(),
                selection: RecordSelection {
                    schema: self.schema.clone(),
                    scope: None,
                    item: None,
                    source_start: None,
                    source_end: None,
                },
                after: 0,
            })
            .await?;
        while let Some(page) = rows.next().await? {
            let page = page.try_map(|rows| -> Result<_> {
                let mut entries = Vec::with_capacity(rows.len());
                for row in rows {
                    let current = row.scope.as_ref() == Some(&self.epoch);
                    let record = row.payload.entry()?;
                    if !current || record.sequence <= through {
                        entries.push(record);
                    }
                }
                Ok(entries)
            })?;
            visit(page)?;
        }
        Ok(())
    }

    pub(super) async fn history_scoped(
        self: &Arc<Self>, session: &crate::storage::Manager, state_manager: &crate::app::StateManager, start: Start, through: u64,
        scope: Option<harmos::Identifier>,
    ) -> Result<History> {
        self.wait_through(state_manager, through).await?;
        let at = self.save_accepted(state_manager).await?;
        let (after, sequence_after) = match start {
            Start::Physical(after) => (after, None),
            Start::Logical(after) => {
                if scope.is_none() {
                    return Err(error("logical seek requires a process epoch"));
                }
                (0, (after > 0).then_some(after))
            }
        };
        Ok(History {
            source: self.clone(),
            state_manager: state_manager.clone(),
            rows: self.rows(session, at, scope, after).await?,
            pending: None,
            pending_index: 0,
            sequence_after,
            after,
            through,
            done: false,
        })
    }

    // Save Everything Accepted So Far; One Saved Cut Then Covers The Whole History.
    async fn save_accepted(&self, state_manager: &crate::app::StateManager) -> Result<harmos::storage::StorageCursor> {
        let accepted = self.storage.cursor_accepted();
        let saved = self.storage.flush_until(&accepted).await?;
        self.healthy(state_manager)?;
        Ok(saved)
    }

    async fn rows(
        &self, session: &crate::storage::Manager, at: harmos::storage::StorageCursor, scope: Option<harmos::Identifier>, after: u64,
    ) -> Result<RecordReader<RecordRow>> {
        Ok(session
            .read_records::<RecordRow>(RecordRead {
                at,
                selection: RecordSelection {
                    schema: self.schema.clone(),
                    scope,
                    item: None,
                    source_start: None,
                    source_end: None,
                },
                after,
            })
            .await?)
    }
}

impl History {
    pub(crate) async fn next(&mut self) -> Result<Option<CapturedRecord>> {
        self.source.healthy(&self.state_manager)?;
        while !self.done {
            if let Some(page) = &self.pending
                && let Some(row) = page.get(self.pending_index)
            {
                if row.current_process && row.record.sequence > self.through {
                    // The Next Frontier Must Still Observe This Unacknowledged Row.
                    self.done = true;
                    self.pending = None;
                    return Ok(None);
                }
                if row.position <= self.after {
                    return Err(error("record storage position did not advance"));
                }
                self.after = row.position;
                let record = CapturedRecord {
                    page: page.clone(),
                    index: self.pending_index,
                };
                self.pending_index += 1;
                return Ok(Some(record));
            }
            self.pending = None;
            self.pending_index = 0;
            let Some(rows) = self.rows.next().await? else {
                self.done = true;
                break;
            };
            let sequence_after = self.sequence_after;
            self.pending = Some(rows.try_map(|rows| -> Result<_> {
                let mut selected = Vec::new();
                selected.try_reserve_exact(rows.len()).map_err(|_| error("captured record allocation failed"))?;
                for row in rows {
                    if sequence_after.is_some_and(|after| row.payload.sequence() <= after) {
                        continue;
                    }
                    selected.push(CapturedRow {
                        position: row.position.sequence,
                        current_process: row.scope.as_ref() == Some(&self.source.epoch),
                        record: row.payload.entry()?,
                    });
                }
                Ok(selected)
            })?);
        }
        Ok(None)
    }
}

impl CapturedRecord {
    pub(crate) fn position(&self) -> u64 {
        self.page[self.index].position
    }
}

impl Deref for CapturedRecord {
    type Target = RecordEntry;

    fn deref(&self) -> &RecordEntry {
        &self.page[self.index].record
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
use std::{collections::VecDeque, ops::Deref, sync::Arc};
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
/// Captured records in storage order, read from one saved cut that covers
/// every record accepted when the history opened.
pub(crate) struct History {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    rows: RecordReader<RecordRow>,
    pending: VecDeque<CapturedRecord>,
    // Logical Seek: Only Rows Journaled After This Capture Sequence.
    sequence_after: Option<u64>,
    after: u64,
    through: u64,
    done: bool,
}

#[derive(Clone)]
pub struct CapturedRecord {
    position: u64,
    current_process: bool,
    record: Arc<RecordEntry>,
}

/* === Enums === */
pub(super) enum Start {
    Physical(u64),
    Logical(u64),
}

/* === Traits === */

/* === Implementations === */
impl Source {
    /// Saved records `select` accepts, excluding this process's records after `through`.
    pub(crate) async fn saved_filtered_records(
        &self, state_manager: &crate::app::StateManager, through: u64, select: impl Fn(&RecordEntry) -> bool,
    ) -> Result<Vec<RecordEntry>> {
        self.wait_through(state_manager, through).await?;
        let at = self.save_accepted(state_manager).await?;
        let rows = self.rows(at, None, 0).await?;
        let mut selected = Vec::new();
        for row in rows.collect().await? {
            let current = row.scope.as_ref() == Some(&self.epoch);
            let record = row.payload.entry()?;
            if (!current || record.sequence <= through) && select(&record) {
                selected.push(record);
            }
        }
        Ok(selected)
    }

    pub(crate) async fn history(self: &Arc<Self>, state_manager: &crate::app::StateManager, after: u64, through: u64) -> Result<History> {
        self.history_scoped(state_manager, Start::Physical(after), through, None).await
    }

    pub(super) async fn history_scoped(
        self: &Arc<Self>, state_manager: &crate::app::StateManager, start: Start, through: u64, scope: Option<harmos::Identifier>,
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
            rows: self.rows(at, scope, after).await?,
            pending: VecDeque::new(),
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
        &self, at: harmos::storage::StorageCursor, scope: Option<harmos::Identifier>, after: u64,
    ) -> Result<RecordReader<RecordRow>> {
        Ok(self
            .storage
            .read_records::<RecordRow>(RecordRead {
                at,
                selection: RecordSelection { schema: self.schema.clone(), scope, item: None, source_start: None, source_end: None },
                after,
            })
            .await?)
    }
}

impl History {
    pub(crate) async fn next(&mut self) -> Result<Option<CapturedRecord>> {
        self.source.healthy(&self.state_manager)?;
        while !self.done {
            if let Some(row) = self.pending.pop_front() {
                if row.current_process && row.record.sequence > self.through {
                    // Physical admission preserves current-process journal order.
                    // Do not acknowledge this row: the next frontier still needs it.
                    self.done = true;
                    self.pending.clear();
                    return Ok(None);
                }
                if row.position <= self.after {
                    return Err(error("record storage position did not advance"));
                }
                self.after = row.position;
                return Ok(Some(row));
            }
            let Some(rows) = self.rows.next().await? else {
                self.done = true;
                break;
            };
            let sequence_after = self.sequence_after;
            for row in rows.into_iter().filter(|row| sequence_after.is_none_or(|after| row.payload.sequence() > after)) {
                self.pending.push_back(CapturedRecord {
                    position: row.position.sequence,
                    current_process: row.scope.as_ref() == Some(&self.source.epoch),
                    record: Arc::new(row.payload.entry()?),
                });
            }
        }
        Ok(None)
    }
}

impl CapturedRecord {
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
}

impl Deref for CapturedRecord {
    type Target = RecordEntry;

    fn deref(&self) -> &RecordEntry {
        &self.record
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

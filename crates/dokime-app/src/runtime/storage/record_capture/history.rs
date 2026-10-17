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
use harmos::storage::{
    RecordRead, RecordReader,
    query::{
        RecordSelection,
        scalar::ScalarValue,
        transform::{
            FieldPath,
            filter::{Clause, Compare, Condition, Filter, Term},
        },
    },
};
// Local Imports
use super::{Source, error};
use crate::{Result, records::RecordEntry};
use dokime::storage::records::RecordRow;

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
    /// Saved records the filter selects, excluding this process's records after `through`.
    pub(crate) async fn saved_filtered_records(
        &self,
        state_manager: &crate::app::StateManager,
        through: u64,
        filter: Filter,
    ) -> Result<Vec<RecordEntry>> {
        self.wait_through(state_manager, through).await?;
        let at = self.save_accepted(state_manager).await?;
        let rows = self.rows(at, None, 0, vec![filter]).await?;
        let mut selected = Vec::new();
        for row in rows.collect().await? {
            let current = row.scope.as_ref() == Some(&self.epoch);
            let record = row.payload.entry()?;
            if !current || record.sequence <= through {
                selected.push(record);
            }
        }
        Ok(selected)
    }

    pub(crate) async fn history(
        self: &Arc<Self>,
        state_manager: &crate::app::StateManager,
        after: u64,
        through: u64,
    ) -> Result<History> {
        self.history_scoped(state_manager, Start::Physical(after), through, None)
            .await
    }

    pub(super) async fn history_scoped(
        self: &Arc<Self>,
        state_manager: &crate::app::StateManager,
        start: Start,
        through: u64,
        scope: Option<harmos::Identifier>,
    ) -> Result<History> {
        self.wait_through(state_manager, through).await?;
        let at = self.save_accepted(state_manager).await?;
        let (after, filters) = match start {
            Start::Physical(after) => (after, Vec::new()),
            Start::Logical(after) => {
                if scope.is_none() {
                    return Err(error("logical seek requires a process epoch"));
                }
                (0, logical_after(after).into_iter().collect())
            }
        };
        Ok(History {
            source: self.clone(),
            state_manager: state_manager.clone(),
            rows: self.rows(at, scope, after, filters).await?,
            pending: VecDeque::new(),
            after,
            through,
            done: false,
        })
    }

    // Save Everything Accepted So Far; One Saved Cut Then Covers The Whole History.
    async fn save_accepted(
        &self,
        state_manager: &crate::app::StateManager,
    ) -> Result<harmos::storage::StorageCursor> {
        let accepted = self.storage.cursor_accepted();
        let saved = self.storage.flush_until(&accepted).await?;
        self.healthy(state_manager)?;
        Ok(saved)
    }

    async fn rows(
        &self,
        at: harmos::storage::StorageCursor,
        scope: Option<harmos::Identifier>,
        after: u64,
        filters: Vec<Filter>,
    ) -> Result<RecordReader<RecordRow>> {
        Ok(self
            .storage
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
                filters,
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
            for row in rows {
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
fn logical_after(after: u64) -> Option<Filter> {
    (after > 0).then(|| Filter {
        clauses: vec![Clause {
            terms: vec![Term {
                field: FieldPath::from("payload.sequence"),
                condition: Condition::Compare {
                    op: Compare::Gt,
                    value: ScalarValue::Unsigned(after),
                },
                negate: false,
            }],
        }],
    })
}

/* === Tests === */

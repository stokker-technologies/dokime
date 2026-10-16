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
    StoredRecord,
    query::{
        Query, RecordSelection, RowSlice, Source as QuerySource,
        scalar::ScalarValue,
        transform::{
            FieldPath, Transform,
            filter::{Clause, Compare, Condition, Filter, Term},
        },
    },
};
// Local Imports
use super::{Source, error};
use crate::{Result, records::RecordEntry};
use dokime::storage::{records, rows::Rows};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Captured records in storage order, read from one saved cut that covers
/// every record accepted when the history opened.
pub(crate) struct History {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    rows: Rows<StoredRecord>,
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
        let rows = self.rows(at, None, vec![Transform::Filter(filter)]).await?;
        let mut selected = Vec::new();
        for row in rows.collect().await? {
            let current = row.scope.as_ref() == Some(&self.epoch);
            let record = records::RecordRow::decode_entry(&row.payload)?;
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
        let (after, transform) = match start {
            Start::Physical(after) => (after, after_term("_storage.position", after)),
            Start::Logical(after) => {
                if scope.is_none() {
                    return Err(error("logical seek requires a process epoch"));
                }
                (0, after_term("payload.sequence", after))
            }
        };
        let transforms = transform
            .map(|term| {
                Transform::Filter(Filter {
                    clauses: vec![Clause { terms: vec![term] }],
                })
            })
            .into_iter()
            .collect();
        Ok(History {
            source: self.clone(),
            state_manager: state_manager.clone(),
            rows: self.rows(at, scope, transforms).await?,
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
        transforms: Vec<Transform>,
    ) -> Result<Rows<StoredRecord>> {
        let query = Query {
            at,
            source: QuerySource::Records(RecordSelection {
                schema: self.schema.clone(),
                scope,
                item: None,
                source_start: None,
                source_end: None,
            }),
            transforms,
            slice: RowSlice::default(),
        };
        Ok(Rows::with_query(
            &self.storage,
            query,
            dokime::storage::rows::graph::<records::RecordRow>()?,
        )
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
                    record: Arc::new(records::RecordRow::decode_entry(&row.payload)?),
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
fn after_term(path: &str, after: u64) -> Option<Term> {
    (after > 0).then(|| Term {
        field: FieldPath::from(path),
        condition: Condition::Compare {
            op: Compare::Gt,
            value: ScalarValue::Unsigned(after),
        },
        negate: false,
    })
}

/* === Tests === */

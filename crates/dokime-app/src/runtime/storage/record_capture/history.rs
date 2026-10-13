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
use std::{num::NonZeroUsize, ops::Deref, sync::Arc};
// Third Party Imports
use harmos::MemorySize;
use harmos::storage::{
    QueryOptions, QueryReplay, QueryValue, ReadLimits, RecordClause, RecordFilter, RecordQuery,
    RecordQueryResult, SharedQueryValue, StorageView, StoredRecord,
    frame::{Compare, Expr, Predicate, ScalarField, ScalarPath},
    reduce::{ScalarKind, ScalarValue},
};
// Local Imports
use super::{FRAME_BYTES, Source, error};
use crate::{Error, Result, records::RecordEntry};
use dokime::storage::records;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct History {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    _view: StorageView,
    saved: RecordQueryResult<records::RecordRow>,
    saved_pending: Option<QueryValue<Vec<Row>>>,
    seek: Option<LogicalSeek>,
    scope: Option<harmos::Identifier>,
    tail: QueryReplay<StoredRecord, StoredRecord<records::RecordRow>>,
    after: u64,
    through: u64,
    saved_done: bool,
    done: bool,
    rows: Option<SharedQueryValue<Vec<Row>>>,
    row_index: usize,
}

#[derive(Clone)]
struct Row {
    position: u64,
    current_process: bool,
    record: RecordEntry,
}

struct LogicalSeek {
    reader: RecordQueryResult<records::RecordRow>,
    after: u64,
    position: u64,
    complete: bool,
}

struct Seek {
    position: u64,
    stopped: bool,
}

#[derive(Clone)]
pub struct CapturedRecord {
    rows: SharedQueryValue<Vec<Row>>,
    index: usize,
}

/* === Enums === */
pub(super) enum Start {
    Physical(u64),
    Logical(u64),
}

/* === Traits === */

/* === Implementations === */
impl Source {
    fn record_query(
        &self,
        scope: Option<&harmos::Identifier>,
        after: u64,
    ) -> RecordQuery<records::RecordRow> {
        let mut query = RecordQuery::new(self.schema.clone(), |bytes, allowance| {
            let memory = records::RecordRow::decoding_memory(bytes).map_err(decoding_error)?;
            if memory == usize::MAX || memory > allowance.get() {
                return Err(exhausted());
            }
            records::RecordRow::from_bytes(bytes, FRAME_BYTES.min(allowance.get()))
                .map_err(decoding_error)
        });
        query.scope = scope.cloned();
        query.options = QueryOptions {
            after,
            page: Some(ReadLimits {
                rows: self.reads.page_rows,
                bytes: self.reads.page_bytes,
            }),
            execution: Some(self.reads.execution),
            decoded_bytes: Some(self.reads.decoded_bytes),
            ..QueryOptions::default()
        };
        query
    }

    pub(crate) async fn saved_filtered_records<T: MemorySize>(
        &self,
        state_manager: &crate::app::StateManager,
        through: u64,
        mut filter: RecordFilter,
        project: impl FnOnce(&[QueryValue<Vec<RecordEntry>>], NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<T> {
        self.wait_through(state_manager, through).await?;
        filter.check()?;
        let predicates = filter
            .clauses
            .iter()
            .map(|clause| clause.predicates.len())
            .sum::<usize>();
        if predicates
            .saturating_mul(2)
            .saturating_add(filter.clauses.len().saturating_mul(3))
            > 64
            || filter.columns.len() > 62
        {
            return Err(harmos::Error::ResourceExhausted {
                resource: "record view frontier predicates",
            }
            .into());
        }
        let epoch_column = filter.columns.len();
        filter.columns.push(ScalarField {
            path: ScalarPath::Scope,
            kind: ScalarKind::Text,
        });
        let sequence_column = filter.columns.len();
        filter.columns.push(ScalarField {
            path: ScalarPath::Payload(vec![
                harmos::Identifier::new("sequence").expect("constant record sequence"),
            ]),
            kind: ScalarKind::Unsigned,
        });
        let original = std::mem::take(&mut filter.clauses);
        for clause in original {
            let mut prior = clause.predicates.clone();
            prior.push(Predicate::Compare {
                expr: Expr::Col(epoch_column),
                compare: Compare::Ne,
                value: ScalarValue::Text(self.epoch.as_str().to_owned()),
            });
            filter.clauses.push(RecordClause { predicates: prior });
            let mut current = clause.predicates;
            current.push(Predicate::Compare {
                expr: Expr::Col(epoch_column),
                compare: Compare::Eq,
                value: ScalarValue::Text(self.epoch.as_str().to_owned()),
            });
            current.push(Predicate::Compare {
                expr: Expr::Col(sequence_column),
                compare: Compare::Le,
                value: ScalarValue::Unsigned(through),
            });
            filter.clauses.push(RecordClause {
                predicates: current,
            });
        }
        filter.check()?;

        // Save One Explicit Accepted Cut; Never Follow A Filtered Live Tail.
        let boundary_view = self.storage.view_with_limits(self.reads.execution).await?;
        let cut = boundary_view.accepted()?;
        drop(boundary_view);
        self.storage.flush_until(&cut).await?;
        self.healthy(state_manager)?;
        let view = self.storage.view_with_limits(self.reads.execution).await?;
        if !view.saved()?.covers(&cut)? {
            return Err(error(
                "filtered record cut is not saved in its original history",
            ));
        }
        let computation = view.begin_computation()?;
        let mut query = self.record_query(None, 0);
        query.options.through = Some(cut.sequence);
        query.filter = Some(filter);
        let mut reader = self.storage.query_in(&view, query)?;
        reader.retain_view()?;
        let mut pages = Vec::new();
        while let Some(page) = reader
            .next_page_projected(self.reads.decoded_bytes, project_saved_records)
            .await?
        {
            computation.check()?;
            pages.try_reserve(1).map_err(|_| exhausted())?;
            if pages
                .capacity()
                .saturating_mul(std::mem::size_of::<QueryValue<Vec<RecordEntry>>>())
                > self.reads.decoded_bytes.get()
            {
                return Err(exhausted().into());
            }
            // Retained Page Claims Stay Charged Until The View Frame Is Built.
            pages.push(page);
        }
        computation.check()?;
        let output = project(&pages, self.reads.decoded_bytes)?;
        if output.memory_size() > self.reads.decoded_bytes.get() {
            return Err(exhausted().into());
        }
        computation.check()?;
        Ok(output)
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
        let view = self
            .storage
            .view_with_limits(self.reads.execution)
            .await
            .map_err(Error::from)?;
        let saved = view.saved().map_err(Error::from)?;
        let (after, seek) = match start {
            Start::Physical(after) => (after, None),
            Start::Logical(after) => {
                let scope = scope
                    .as_ref()
                    .ok_or_else(|| error("logical seek requires a process epoch"))?;
                let query = self.record_query(Some(scope), 0);
                let reader = self.storage.query_in(&view, query).map_err(Error::from)?;
                (
                    0,
                    Some(LogicalSeek {
                        reader,
                        after,
                        position: 0,
                        complete: false,
                    }),
                )
            }
        };
        // Replay Retains The Exact Accepted Cut, Including Writes Saved During Seek
        let accepted = view.accepted().map_err(Error::from)?;
        let mut query = self.record_query(scope.as_ref(), after.max(saved.sequence));
        query.options.through = Some(accepted.sequence);
        let tail = self.storage.replay(query).await.map_err(Error::from)?;
        if tail.boundary() != &accepted {
            return Err(error(
                "record replay changed its original accepted boundary",
            ));
        }

        let query = self.record_query(scope.as_ref(), after.min(saved.sequence));
        let mut saved_reader = self.storage.query_in(&view, query).map_err(Error::from)?;
        saved_reader.retain_view().map_err(Error::from)?;
        Ok(History {
            source: self.clone(),
            state_manager: state_manager.clone(),
            _view: view,
            saved_done: after >= saved.sequence,
            saved: saved_reader,
            saved_pending: None,
            seek,
            scope,
            tail,
            after,
            through,
            done: false,
            rows: None,
            row_index: 0,
        })
    }
}

impl History {
    pub(crate) async fn next(&mut self) -> Result<Option<CapturedRecord>> {
        self.source.healthy(&self.state_manager)?;
        self.resolve_seek().await?;
        while !self.done {
            if self
                .rows
                .as_ref()
                .is_some_and(|rows| self.row_index < rows.len())
            {
                let row = &self.rows.as_ref().unwrap()[self.row_index];
                if row.current_process && row.record.sequence > self.through {
                    // Physical admission preserves current-process journal order.
                    // Do not acknowledge this row: the next frontier still needs it.
                    self.done = true;
                    self.rows = None;
                    return Ok(None);
                }
                if row.position <= self.after {
                    return Err(error("record storage position did not advance"));
                }
                self.after = row.position;
                let record = CapturedRecord {
                    rows: self.rows.as_ref().unwrap().clone(),
                    index: self.row_index,
                };
                self.row_index += 1;
                return Ok(Some(record));
            }
            self.rows = None;
            self.row_index = 0;
            if !self.saved_done {
                self.read_saved().await?;
            } else {
                self.read_tail().await?;
            }
        }
        Ok(None)
    }

    async fn resolve_seek(&mut self) -> Result<()> {
        let Some(seek) = &mut self.seek else {
            return Ok(());
        };
        while !seek.complete {
            let position = seek.position;
            let after = seek.after;
            let selected = seek
                .reader
                .next_page_projected(self.source.reads.decoded_bytes, |rows, allowance| {
                    seek_saved(rows, after, position, allowance)
                })
                .await
                .map_err(Error::from)?;
            let Some(selected) = selected else {
                seek.complete = true;
                break;
            };
            seek.position = selected.position;
            seek.complete = selected.stopped;
        }
        let position = seek.position;
        let query = self.source.record_query(self.scope.as_ref(), position);
        let mut reader = self
            .source
            .storage
            .query_in(&self._view, query)
            .map_err(Error::from)?;
        reader.retain_view().map_err(Error::from)?;
        self.saved = reader;
        self.after = position;
        self.saved_done = position >= self._view.saved().map_err(Error::from)?.sequence;
        self.seek = None;
        Ok(())
    }

    async fn read_saved(&mut self) -> Result<()> {
        if self.saved_pending.is_none() {
            let epoch = &self.source.epoch;
            self.saved_pending = self
                .saved
                .next_page_projected(self.source.reads.decoded_bytes, |rows, allowance| {
                    project_records(rows, epoch, allowance)
                })
                .await
                .map_err(Error::from)?;
        }
        let Some(pending) = &self.saved_pending else {
            self.saved_done = true;
            return Ok(());
        };
        // A Refused Shared Header Keeps The Exact Derived Page For Retry
        let copied = self
            .saved
            .project(
                pending,
                self.source.reads.decoded_bytes,
                |rows, allowance| copy_rows(rows, allowance),
            )
            .map_err(Error::from)?;
        let computation = self._view.begin_computation().map_err(Error::from)?;
        let shared = copied.try_into_shared().map_err(Error::from)?;
        computation.check().map_err(Error::from)?;
        drop(computation);
        self.saved_pending = None;
        self.rows = Some(shared);
        self.row_index = 0;
        Ok(())
    }

    async fn read_tail(&mut self) -> Result<()> {
        let epoch = &self.source.epoch;
        let decoded = self
            .tail
            .next_page_projected_shared(self.source.reads.decoded_bytes, |rows, allowance| {
                project_records(rows, epoch, allowance)
            })
            .await;
        let decoded = match decoded {
            Err(harmos::Error::Gap { .. }) => {
                // Writer pressure can release the live tail after saving it.
                // Reopen the same immutable cut at the last delivered position;
                // never advance the frontier or silently skip the missing range.
                self.source
                    .storage
                    .flush_until(self.tail.boundary())
                    .await
                    .map_err(Error::from)?;
                let mut query = self.source.record_query(self.scope.as_ref(), self.after);
                query.options.through = Some(self.tail.boundary().sequence);
                self.tail = self
                    .source
                    .storage
                    .replay(query)
                    .await
                    .map_err(Error::from)?;
                self.tail
                    .next_page_projected_shared(
                        self.source.reads.decoded_bytes,
                        |rows, allowance| project_records(rows, epoch, allowance),
                    )
                    .await
                    .map_err(Error::from)?
            }
            result => result.map_err(Error::from)?,
        };
        let Some(decoded) = decoded else {
            self.done = true;
            return Ok(());
        };
        self.rows = Some(decoded);
        self.row_index = 0;
        Ok(())
    }
}

impl CapturedRecord {
    pub(crate) fn position(&self) -> u64 {
        self.rows[self.index].position
    }

    pub fn try_project<U: MemorySize>(
        &self,
        max_bytes: NonZeroUsize,
        project: impl FnOnce(&RecordEntry, NonZeroUsize) -> harmos::Result<U>,
    ) -> harmos::Result<QueryValue<U>> {
        self.rows.try_project(max_bytes, |rows, allowance| {
            project(&rows[self.index].record, allowance)
        })
    }
}

impl Deref for CapturedRecord {
    type Target = RecordEntry;

    fn deref(&self) -> &RecordEntry {
        &self.rows[self.index].record
    }
}

impl MemorySize for Seek {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl MemorySize for Row {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.record
                .memory_size()
                .saturating_sub(std::mem::size_of::<RecordEntry>()),
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn seek_saved(
    rows: &[StoredRecord<records::RecordRow>],
    after: u64,
    position: u64,
    allowance: NonZeroUsize,
) -> harmos::Result<Seek> {
    let mut selected = Seek {
        position,
        stopped: false,
    };
    for row in rows {
        let memory = construction_memory(&row.payload)?
            .checked_add(std::mem::size_of::<Seek>())
            .ok_or_else(exhausted)?;
        if memory > allowance.get() {
            return Err(exhausted());
        }
        let decoded = row
            .payload
            .clone()
            .into_entry(allowance)
            .map_err(decoding_error)?;
        if decoded.sequence > after {
            selected.stopped = true;
            break;
        }
        selected.position = row.position.sequence;
    }
    Ok(selected)
}

fn project_saved_records(
    rows: &[StoredRecord<records::RecordRow>],
    allowance: NonZeroUsize,
) -> harmos::Result<Vec<RecordEntry>> {
    let slots = rows
        .len()
        .checked_mul(std::mem::size_of::<RecordEntry>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<RecordEntry>>()))
        .ok_or_else(exhausted)?;
    let memory = rows.iter().try_fold(slots, |bytes, row| {
        bytes
            .checked_add(construction_memory(&row.payload)?)
            .ok_or_else(exhausted)
    })?;
    if memory > allowance.get() {
        return Err(exhausted());
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(rows.len())
        .map_err(|_| exhausted())?;
    if output.capacity() != rows.len() {
        return Err(exhausted());
    }
    for row in rows {
        output.push(
            row.payload
                .clone()
                .into_entry(allowance)
                .map_err(decoding_error)?,
        );
    }
    Ok(output)
}

fn project_records(
    rows: &[StoredRecord<records::RecordRow>],
    epoch: &harmos::Identifier,
    allowance: NonZeroUsize,
) -> harmos::Result<Vec<Row>> {
    let mut memory = rows
        .len()
        .checked_mul(std::mem::size_of::<Row>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<Row>>()))
        .ok_or_else(exhausted)?;
    for row in rows {
        memory = memory
            .checked_add(construction_memory(&row.payload)?)
            .ok_or_else(exhausted)?;
    }
    if memory > allowance.get() {
        return Err(exhausted());
    }
    let mut projected = Vec::new();
    projected
        .try_reserve_exact(rows.len())
        .map_err(|_| exhausted())?;
    if projected.capacity() != rows.len() {
        return Err(exhausted());
    }
    for row in rows {
        let record = row
            .payload
            .clone()
            .into_entry(allowance)
            .map_err(decoding_error)?;
        projected.push(Row {
            position: row.position.sequence,
            current_process: row.scope.as_ref() == Some(epoch),
            record,
        });
    }
    Ok(projected)
}

fn copy_rows(rows: &[Row], allowance: NonZeroUsize) -> harmos::Result<Vec<Row>> {
    let memory = rows
        .iter()
        .try_fold(std::mem::size_of::<Vec<Row>>(), |bytes, row| {
            bytes.checked_add(row.memory_size()).ok_or_else(exhausted)
        })?;
    if memory == usize::MAX || memory > allowance.get() {
        return Err(exhausted());
    }
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(rows.len())
        .map_err(|_| exhausted())?;
    if copied.capacity() != rows.len() {
        return Err(exhausted());
    }
    copied.extend(rows.iter().cloned());
    Ok(copied)
}

fn construction_memory(row: &records::RecordRow) -> harmos::Result<usize> {
    let native = row.memory_size();
    let projected = row.projection_memory().map_err(decoding_error)?;
    if native == usize::MAX || projected == usize::MAX {
        return Err(exhausted());
    }
    native.checked_add(projected).ok_or_else(exhausted)
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "capture history construction memory",
    }
}

fn decoding_error(source: dokime::storage::Error) -> harmos::Error {
    harmos::Error::backend(
        "decode captured record",
        harmos::BackendFailure::Permanent,
        source,
    )
}

/* === Tests === */

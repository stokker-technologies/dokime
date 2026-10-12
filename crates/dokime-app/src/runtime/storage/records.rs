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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{QueryOptions, QueryValue, ReadLimits, RecordQuery, RecordQueryResult, StoredRecord},
};

// Local Imports
use super::{context_records::ContextRecord, execution::ExecutionView};
use crate::{Error, Result};
use dokime::{
    core::{CaseId, RecordId, SessionId, TestCycleId, TestExecutionId},
    records::{RecordEntry, TestRecordSet},
    storage::{Manager, records},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type PositionedRecord = super::paging::Positioned<ContextRecord>;

/* === Structs === */
pub struct RecordReader {
    reader: RecordQueryResult<records::RecordRow>,
    view: ExecutionView,
    context: bool,
    projection: NonZeroUsize,
    pending: Option<QueryValue<Vec<PositionedRecord>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RecordReader {
    pub(super) fn open_page(storage: &Manager, view: &ExecutionView, after: u64) -> Result<Self> {
        let query = selected_query(
            view,
            QueryOptions {
                after,
                ..QueryOptions::default()
            },
        )?;
        let mut reader = storage.query_in(view.view(), query)?;
        reader.retain_view()?;
        let projection = reader.decoded_bytes();
        Ok(Self {
            reader,
            view: view.clone(),
            context: false,
            projection,
            pending: None,
        })
    }

    pub(super) fn allowance(&self) -> NonZeroUsize {
        self.projection
    }

    pub(super) fn project<T: MemorySize, U: MemorySize>(
        &mut self,
        value: &QueryValue<T>,
        map: impl FnOnce(&T, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        Ok(self.reader.project(value, self.projection, map)?)
    }
    pub(crate) fn open(
        storage: &Manager,
        view: &ExecutionView,
        page: ReadLimits,
        projection: NonZeroUsize,
        context: bool,
    ) -> Result<Self> {
        let query = selected_query(
            view,
            QueryOptions {
                page: Some(page),
                decoded_bytes: Some(projection),
                ..QueryOptions::default()
            },
        )?;
        let mut reader = storage.query_in(view.view(), query)?;
        reader.retain_view()?;
        Ok(Self {
            reader,
            view: view.clone(),
            context,
            projection,
            pending: None,
        })
    }

    pub async fn next(&mut self) -> Result<Option<QueryValue<Vec<ContextRecord>>>> {
        if self.pending.is_none() {
            self.pending = self.next_positioned().await?;
        }
        let Some(positioned) = &self.pending else {
            return Ok(None);
        };
        let projected = self
            .reader
            .project(positioned, self.projection, |rows, allowance| {
                let memory = rows
                    .iter()
                    .try_fold(std::mem::size_of::<Vec<ContextRecord>>(), |bytes, row| {
                        bytes.checked_add(row.captured.memory_size())
                    })
                    .ok_or_else(exhausted)?;
                if memory > allowance.get() {
                    return Err(exhausted());
                }
                let mut records = Vec::new();
                records
                    .try_reserve_exact(rows.len())
                    .map_err(|_| exhausted())?;
                if records.capacity() > rows.len() {
                    return Err(exhausted());
                }
                records.extend(rows.iter().map(|row| row.captured.clone()));
                Ok(records)
            })?;
        self.pending = None;
        Ok(Some(projected))
    }

    pub(super) async fn next_positioned(
        &mut self,
    ) -> Result<Option<QueryValue<Vec<PositionedRecord>>>> {
        if self.pending.is_some() {
            return Err(invalid("record reader has an unsettled context projection"));
        }
        let execution = self.view.execution();
        let context = self.context;
        let selected = self
            .reader
            .next_page_projected(self.projection, |rows, allowance| {
                let slots = rows
                    .len()
                    .checked_mul(std::mem::size_of::<PositionedRecord>())
                    .and_then(|bytes| {
                        bytes.checked_add(std::mem::size_of::<Vec<PositionedRecord>>())
                    })
                    .ok_or_else(|| {
                        codec_error(dokime::storage::Error::Invalid(
                            "record page slot memory overflow",
                        ))
                    })?;
                let native_bytes = rows
                    .iter()
                    .try_fold(slots, |bytes, row| {
                        bytes
                            .checked_add(row.payload.projection_memory()?)
                            .and_then(|bytes| bytes.checked_add(row.payload.memory_size()))
                            .and_then(|bytes| bytes.checked_add(row.item.memory_size()))
                            .and_then(|bytes| bytes.checked_add(row.scope.memory_size()))
                            .ok_or(dokime::storage::Error::Invalid(
                                "record page projection memory overflow",
                            ))
                    })
                    .map_err(codec_error)?;
                // Lookup Scratch And Frozen Scope Copies Share The Original Claim
                let bytes = rows.iter().try_fold(native_bytes, |bytes, row| {
                    let id = RecordId::from_uuid(row.item.as_str())
                        .map_err(|source| codec_error(source.into()))?;
                    let scope = execution.records.subscribed_records().get(&id);
                    bytes
                        .checked_add(scope.map_or(0, MemorySize::memory_size))
                        .ok_or_else(|| {
                            codec_error(dokime::storage::Error::Invalid(
                                "record capture scope memory overflow",
                            ))
                        })
                })?;
                if bytes > allowance.get() {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "captured record projection memory",
                    });
                }
                let mut selected = Vec::with_capacity(rows.len());
                let unused = selected
                    .capacity()
                    .saturating_sub(rows.len())
                    .checked_mul(std::mem::size_of::<PositionedRecord>())
                    .and_then(|unused| bytes.checked_add(unused));
                if unused.is_none_or(|bytes| bytes > allowance.get()) {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "captured record page capacity",
                    });
                }
                for row in rows {
                    let mut decoded = decode(row, allowance)?;
                    if select_execution(
                        &mut decoded.record,
                        &execution.records,
                        &execution.test_cycle_id,
                        &execution.test_case_id,
                        &execution.test_execution_id,
                    )? || (context
                        && super::context_records::matches(
                            Some(&execution.test_cycle_id),
                            decoded.record.scope.as_ref(),
                        ))
                    {
                        selected.push(PositionedRecord {
                            position: row.position.sequence,
                            captured: decoded,
                        });
                    }
                }
                Ok(selected)
            })
            .await?;
        Ok(selected)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn selected_query(
    view: &ExecutionView,
    mut options: QueryOptions,
) -> Result<RecordQuery<records::RecordRow>> {
    let through = view.evidence_through()?;
    if !view.view().saved()?.covers(through)? {
        return Err(invalid("record evidence exceeds the original saved view"));
    }
    options.through = Some(through.sequence);
    let mut query = RecordQuery::new(records::schema(), |bytes, allowance| {
        let bound = records::RecordRow::decoding_memory(bytes).map_err(codec_error)?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(harmos::Error::ResourceExhausted {
                resource: "native record decoding memory",
            });
        }
        records::RecordRow::from_bytes(bytes, allowance.get()).map_err(codec_error)
    });
    query.options = options;
    Ok(query)
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "captured record projection memory",
    }
}
fn decode(
    row: &StoredRecord<records::RecordRow>,
    allowance: NonZeroUsize,
) -> harmos::Result<ContextRecord> {
    let epoch = row.scope.as_ref().ok_or_else(|| {
        codec_error(dokime::storage::Error::Invalid(
            "captured record has no process epoch",
        ))
    })?;
    if epoch.as_str().bytes().any(|byte| byte.is_ascii_uppercase())
        || row.source.is_some()
        || row.source_time.is_some()
    {
        return Err(codec_error(dokime::storage::Error::Invalid(
            "captured record envelope differs from its declared schema",
        )));
    }
    let epoch =
        SessionId::from_uuid(epoch.as_str()).map_err(|source| codec_error(source.into()))?;
    let record = row
        .payload
        .clone()
        .into_entry(allowance)
        .map_err(codec_error)?;
    if row.item.as_str() != record.record_id.as_str() {
        return Err(codec_error(dokime::storage::Error::Invalid(
            "captured record key differs from payload identity",
        )));
    }
    Ok(ContextRecord { epoch, record })
}

fn select_execution(
    record: &mut RecordEntry,
    captures: &TestRecordSet,
    cycle: &TestCycleId,
    case: &CaseId,
    execution: &TestExecutionId,
) -> harmos::Result<bool> {
    if let Some(scope) = captures.subscribed_records().get(&record.record_id) {
        if scope.test_execution_id.as_ref() != Some(execution)
            || scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle)
            || scope.test_case_id.as_ref().is_some_and(|id| id != case)
        {
            return Err(codec_error(dokime::storage::Error::Invalid(
                "frozen record subscription belongs to another execution",
            )));
        }
        record.scope = Some(scope.clone());
    }
    let Some(scope) = record.scope.as_ref() else {
        return Ok(false);
    };
    if scope.test_execution_id.as_ref() != Some(execution) {
        return Ok(false);
    }
    if scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle)
        || scope.test_case_id.as_ref().is_some_and(|id| id != case)
    {
        return Err(codec_error(dokime::storage::Error::Invalid(
            "captured record belongs to another cycle or case",
        )));
    }
    Ok(true)
}

fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    match source {
        dokime::storage::Error::Runtime(source) => source,
        source => harmos::Error::backend(
            "decode native captured record",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */

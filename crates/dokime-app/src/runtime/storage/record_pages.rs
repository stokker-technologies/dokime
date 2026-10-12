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
use harmos::{MemorySize, storage::QueryValue};

// Local Imports
use super::{
    context_records::ContextRecord,
    execution::ExecutionView,
    paging::{Cursor, State, page_limit, update},
    records::RecordReader,
};
use crate::{
    Result,
    runtime::query::checkpoint_records::{ExecutionRecordPage, ExecutionRecordQuery},
};
use dokime::storage::Manager;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Collected {
    reader: RecordReader,
    page: QueryValue<ExecutionRecordPage>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */
pub(crate) async fn read(
    storage: &Manager,
    view: &ExecutionView,
    query: ExecutionRecordQuery,
) -> Result<QueryValue<ExecutionRecordPage>> {
    Ok(collect(storage, view, query).await?.page)
}

pub(crate) async fn read_projected<U: MemorySize>(
    storage: &Manager,
    view: &ExecutionView,
    query: ExecutionRecordQuery,
    map: impl FnOnce(&ExecutionRecordPage, NonZeroUsize) -> harmos::Result<U>,
) -> Result<QueryValue<U>> {
    let mut collected = collect(storage, view, query).await?;
    collected.reader.project(&collected.page, map)
}

/* === Private Functions === */
async fn collect(
    storage: &Manager,
    view: &ExecutionView,
    query: ExecutionRecordQuery,
) -> Result<Collected> {
    let limit = page_limit(query.limit, "record")?;
    let after = Cursor::requested(query.after.as_deref())?;
    let mut reader = RecordReader::open_page(storage, view, after)?;
    let checkpoint = view.checkpoint(reader.allowance())?;
    let cursor = reader.project(&checkpoint, |checkpoint, _| {
        Cursor::open(checkpoint, "records", "", query.after.as_deref())
            .map_err(|source| source_error(source.into()))
    })?;
    let metadata = std::mem::size_of::<ExecutionRecordPage>()
        .checked_add(
            checkpoint
                .memory_size()
                .saturating_sub(std::mem::size_of_val(&*checkpoint)),
        )
        .and_then(|bytes| bytes.checked_add(Cursor::memory()))
        .ok_or_else(exhausted)?;
    let capacity = reader
        .allowance()
        .get()
        .checked_sub(metadata)
        .ok_or_else(exhausted)?;
    let mut state: Option<QueryValue<State<ContextRecord>>> = None;
    while let Some(rows) = reader.next_positioned().await? {
        let previous = state.as_deref();
        let next = reader.project(&rows, |rows, _| {
            update(previous, rows, after, limit, capacity)
        })?;
        let complete = next.more;
        state = Some(next);
        if complete {
            break;
        }
    }
    let page = reader.project(&checkpoint, |checkpoint, allowance| {
        let records = state
            .as_ref()
            .map_or(&[][..], |state| state.rows.as_slice());
        let memory = records
            .iter()
            .try_fold(metadata, |bytes, record| {
                bytes.checked_add(record.memory_size())
            })
            .ok_or_else(exhausted)?;
        if memory > allowance.get() {
            return Err(exhausted());
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(records.len())
            .map_err(|_| exhausted())?;
        if output.capacity() > records.len() {
            return Err(exhausted());
        }
        output.extend(records.iter().cloned());
        let next = state
            .as_ref()
            .filter(|state| state.more)
            .map(|state| {
                cursor
                    .encode(state.last)
                    .map_err(|source| source_error(source.into()))
            })
            .transpose()?;
        Ok(ExecutionRecordPage {
            checkpoint: checkpoint.clone(),
            records: output,
            next,
            issues: Vec::new(),
            issues_truncated: false,
        })
    })?;
    Ok(Collected { reader, page })
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "native record page",
    }
}

fn source_error(source: crate::Error) -> harmos::Error {
    match source {
        crate::Error::Harmos(source) => source,
        crate::Error::Storage(dokime::storage::Error::Runtime(source)) => source,
        source => harmos::Error::backend(
            "select native record continuation",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

/* === Tests === */

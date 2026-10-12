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
use harmos::{Identifier, MemorySize, storage::QueryValue};

// Local Imports
use super::{
    execution::ExecutionView,
    paging::{Cursor, State, page_limit, update},
    telemetry::TelemetryReader,
};
use crate::{
    Result,
    runtime::query::checkpoint_telemetry::{ExecutionTelemetryPage, ExecutionTelemetryQuery},
};
use dokime::storage::{Manager, telemetry::TelemetryEntry};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Collected {
    reader: TelemetryReader,
    page: QueryValue<ExecutionTelemetryPage>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */
pub(crate) async fn read(
    storage: &Manager,
    view: &ExecutionView,
    stream: &Identifier,
    query: ExecutionTelemetryQuery,
) -> Result<QueryValue<ExecutionTelemetryPage>> {
    Ok(collect(storage, view, stream, query).await?.page)
}

pub(crate) async fn read_projected<U: MemorySize>(
    storage: &Manager,
    view: &ExecutionView,
    stream: &Identifier,
    query: ExecutionTelemetryQuery,
    map: impl FnOnce(&ExecutionTelemetryPage, NonZeroUsize) -> harmos::Result<U>,
) -> Result<QueryValue<U>> {
    let mut collected = collect(storage, view, stream, query).await?;
    collected.reader.project(&collected.page, map)
}

/* === Private Functions === */
async fn collect(
    storage: &Manager,
    view: &ExecutionView,
    stream: &Identifier,
    query: ExecutionTelemetryQuery,
) -> Result<Collected> {
    let limit = page_limit(query.limit, "telemetry")?;
    let after = Cursor::requested(query.after.as_deref())?;
    let mut reader = TelemetryReader::open_page(storage, view, stream, after)?;
    let checkpoint = view.checkpoint(reader.allowance())?;
    let cursor = reader.project(&checkpoint, |checkpoint, _| {
        Cursor::open(
            checkpoint,
            "telemetry",
            stream.as_str(),
            query.after.as_deref(),
        )
        .map_err(|source| source_error(source.into()))
    })?;
    let metadata = std::mem::size_of::<ExecutionTelemetryPage>()
        .checked_add(
            checkpoint
                .memory_size()
                .saturating_sub(std::mem::size_of_val(&*checkpoint)),
        )
        .and_then(|bytes| bytes.checked_add(Cursor::memory()))
        .and_then(|bytes| bytes.checked_add(stream.as_str().len()))
        .ok_or_else(exhausted)?;
    let capacity = reader
        .allowance()
        .get()
        .checked_sub(metadata)
        .ok_or_else(exhausted)?;
    let mut state: Option<QueryValue<State<TelemetryEntry>>> = None;
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
        let rows = state
            .as_ref()
            .map_or(&[][..], |state| state.rows.as_slice());
        let memory = rows
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
            .try_reserve_exact(rows.len())
            .map_err(|_| exhausted())?;
        if output.capacity() > rows.len() {
            return Err(exhausted());
        }
        output.extend(rows.iter().cloned());
        let next = state
            .as_ref()
            .filter(|state| state.more)
            .map(|state| {
                cursor
                    .encode(state.last)
                    .map_err(|source| source_error(source.into()))
            })
            .transpose()?;
        let mut table = String::new();
        table
            .try_reserve_exact(stream.as_str().len())
            .map_err(|_| exhausted())?;
        if table.capacity() > stream.as_str().len() {
            return Err(exhausted());
        }
        table.push_str(stream.as_str());
        Ok(ExecutionTelemetryPage {
            table,
            checkpoint: checkpoint.clone(),
            rows: output,
            next,
            issues: Vec::new(),
            issues_truncated: false,
        })
    })?;
    Ok(Collected { reader, page })
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "native telemetry page",
    }
}

fn source_error(source: crate::Error) -> harmos::Error {
    match source {
        crate::Error::Harmos(source) => source,
        crate::Error::Storage(dokime::storage::Error::Runtime(source)) => source,
        source => harmos::Error::backend(
            "select native telemetry continuation",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

/* === Tests === */

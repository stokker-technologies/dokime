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
    storage::{
        ObjectQuery, ObjectQueryResult, ObjectRevision, ObjectVersionToken, QueryOptions,
        QueryValue, ReadLimits, StorageCursor, StorageView,
    },
};

// Local Imports
use super::execution::check_seal_position;
use crate::{Error, Result};
use dokime::{
    execution::TestExecution,
    storage::{Manager, objects::execution::Execution},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Reader {
    reader: ObjectQueryResult<Execution>,
    view: StorageView,
    projection: NonZeroUsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Reader {
    pub(crate) fn open(
        storage: &Manager,
        view: &StorageView,
        page: ReadLimits,
        projection: NonZeroUsize,
    ) -> Result<Self> {
        let mut query = ObjectQuery::<Execution>::new()
            .with_decoding_memory(|bytes| Execution::decoding_memory(bytes).map_err(codec_error));
        query.options = QueryOptions {
            page: Some(page),
            decoded_bytes: Some(projection),
            ..QueryOptions::default()
        };
        let mut reader = storage.query_in(view, query)?;
        reader.retain_view()?;
        Ok(Self {
            reader,
            view: view.clone(),
            projection,
        })
    }

    pub(crate) async fn next_projected<U: MemorySize>(
        &mut self,
        map: impl FnOnce(&[ObjectRevision<Execution>], NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<Option<QueryValue<U>>> {
        let view = &self.view;
        let selected = self
            .reader
            .next_page_projected(self.projection, |rows, allowance| {
                let saved = view.saved()?;
                for row in rows {
                    row.version.check_position(&row.position)?;
                    if !saved.covers(&row.position)? {
                        return Err(invalid("history revision exceeds its original saved view"));
                    }
                }
                map(rows, allowance)
            })
            .await?;
        Ok(selected)
    }

    pub(crate) fn project<U: MemorySize, V: MemorySize>(
        &mut self,
        input: &QueryValue<U>,
        map: impl FnOnce(&U, NonZeroUsize) -> harmos::Result<V>,
    ) -> Result<QueryValue<V>> {
        Ok(self.reader.project(input, self.projection, map)?)
    }

    pub(crate) async fn next(
        &mut self,
    ) -> Result<Option<QueryValue<Vec<ObjectRevision<TestExecution>>>>> {
        let selected = self
            .next_projected(|rows, allowance| {
                let mut bytes = rows
                    .len()
                    .checked_mul(std::mem::size_of::<ObjectRevision<TestExecution>>())
                    .and_then(|bytes| {
                        bytes.checked_add(std::mem::size_of::<Vec<ObjectRevision<TestExecution>>>())
                    })
                    .ok_or_else(|| invalid("history page slot memory overflows"))?;
                for row in rows {
                    let projected = row.value.projection_memory().map_err(codec_error)?;
                    let native = row.value.memory_size();
                    let position = row.position.memory_size();
                    let version = row.version.memory_size();
                    if [projected, native, position, version].contains(&usize::MAX) {
                        return Err(invalid("history projection memory overflows"));
                    }
                    // Include Native Clone Scratch, Domain Heap And Revision Copies.
                    bytes = bytes
                        .checked_add(native)
                        .and_then(|bytes| {
                            bytes.checked_add(
                                projected.saturating_sub(std::mem::size_of::<TestExecution>()),
                            )
                        })
                        .and_then(|bytes| {
                            bytes.checked_add(
                                position.saturating_sub(std::mem::size_of::<StorageCursor>()),
                            )
                        })
                        .and_then(|bytes| {
                            bytes.checked_add(
                                version.saturating_sub(std::mem::size_of::<ObjectVersionToken>()),
                            )
                        })
                        .ok_or_else(|| invalid("history projection memory overflows"))?;
                }
                if bytes > allowance.get() {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "execution history projection",
                    });
                }
                let mut selected = Vec::new();
                selected.try_reserve_exact(rows.len()).map_err(|_| {
                    harmos::Error::ResourceExhausted {
                        resource: "execution history page slots",
                    }
                })?;
                for row in rows {
                    let execution = row
                        .value
                        .clone()
                        .into_execution(allowance)
                        .map_err(codec_error)?;
                    if let Some(seal) = &execution.saved {
                        check_seal_position(&row.position, &seal.dependencies_saved)
                            .map_err(query_error)?;
                    }
                    selected.push(ObjectRevision {
                        value: execution,
                        position: row.position.clone(),
                        version: row.version.clone(),
                    });
                }
                Ok(selected)
            })
            .await?;
        Ok(selected)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(reason: &'static str) -> harmos::Error {
    codec_error(dokime::storage::Error::Invalid(reason))
}

fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    harmos::Error::backend(
        "decode execution history",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

fn query_error(source: Error) -> harmos::Error {
    harmos::Error::backend(
        "verify execution history",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */

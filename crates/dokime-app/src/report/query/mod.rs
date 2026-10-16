/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod plots;
mod statistics;
pub(crate) mod telemetry;

/* === Re-Exports === */
pub(super) use plots::materialize;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::{
    MemorySize,
    operations::{Context, StreamSettings},
    storage::{
        QueryValue,
        arrow::ArrowBatch,
        query::{Query, QueryOutput},
    },
};
// Local Imports
use crate::{Error, Result};
use dokime::storage::Manager;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub(super) struct ReadContext<'a, 'ctx> {
    pub ctx: &'a Context<'ctx>,
    pub storage: &'a Manager,
    pub settings: &'a crate::config::StorageSettings,
}

pub(super) struct Batches {
    values: Vec<ArrowBatch>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::ops::Deref for Batches {
    type Target = [ArrowBatch];
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl MemorySize for Batches {
    fn memory_size(&self) -> usize {
        // Arrow Allocations Retain Their Own Storage Reservations
        std::mem::size_of::<Self>().saturating_add(
            self.values
                .capacity()
                .saturating_mul(std::mem::size_of::<ArrowBatch>()),
        )
    }
}

/* === Public Functions === */
pub(super) async fn batches(
    ctx: &Context<'_>,
    storage: &Manager,
    settings: &crate::config::StorageSettings,
    query: Query,
) -> Result<QueryValue<Batches>> {
    let at = query.at.clone();
    let session = ctx.download::<harmos::storage::operations::Query>(
        query,
        StreamSettings {
            items: NonZeroUsize::new(1).unwrap(),
            bytes: settings.budget.read_memory_bytes,
        },
    )?;
    let (_, mut output, completion) = session.split();
    let mut schema = None;
    let mut rows = 0u64;
    let mut retained = storage
        .produce(nonzero(std::mem::size_of::<Batches>())?, |_| {
            Ok(Batches { values: Vec::new() })
        })?
        .try_into_value()?;
    while let Some(item) = output.next().await? {
        match &*item {
            QueryOutput::Schema(value) => {
                if schema.is_some() || rows != 0 {
                    return Err(invalid("query emitted a repeated or late schema"));
                }
                schema = Some(value.clone());
            }
            QueryOutput::Batch(batch) => {
                let announced = schema
                    .as_ref()
                    .ok_or_else(|| invalid("query emitted data before its schema"))?;
                batch.validate_schema(announced.as_schema())?;
                rows = rows
                    .checked_add(batch.as_record_batch().num_rows() as u64)
                    .ok_or_else(|| invalid("query row count overflow"))?;
                let bound = std::mem::size_of::<Vec<ArrowBatch>>()
                    .checked_add(
                        (retained.len() + 1)
                            .checked_mul(std::mem::size_of::<ArrowBatch>())
                            .ok_or_else(exhausted)?,
                    )
                    .ok_or_else(exhausted)?;
                retained = retained.try_map(nonzero(bound)?, |mut retained, _| {
                    retained
                        .values
                        .try_reserve_exact(1)
                        .map_err(|_| exhausted())?;
                    retained.values.push(batch.clone());
                    Ok(retained)
                })?;
            }
        }
    }
    let completed = completion.join().await?;
    if schema.is_none() || completed.at != at || completed.rows != rows {
        return Err(invalid(
            "query completion differs from the consumed evidence",
        ));
    }
    Ok(retained)
}

pub(super) fn empty<T: MemorySize>(storage: &Manager) -> Result<QueryValue<Vec<T>>> {
    Ok(storage
        .produce(nonzero(std::mem::size_of::<Vec<T>>())?, |_| Ok(Vec::new()))?
        .try_into_value()?)
}

pub(super) fn nonzero(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes).ok_or_else(|| invalid("report memory bound is empty"))
}
pub(super) fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "report query output memory",
    }
}
pub(super) fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Private Functions === */

/* === Tests === */

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod fields;
mod groups;
mod pages;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectQuery, QueryResult, QueryValue, RecordQuery},
};

// Local Imports
use crate::runtime::query::aggregate::{AggregateOutcome, AggregateQuery, AggregateSource};
use crate::{Error, Result};
use dokime::storage::{
    Manager,
    objects::{case::Case, cycle::Cycle, execution::Execution},
    records,
};
use groups::Groups;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Collected<R, T> {
    reader: QueryResult<R, T>,
    outcome: QueryValue<AggregateOutcome>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for AggregateOutcome {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.columns
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.columns)),
            )
            .saturating_add(
                self.rows
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.rows)),
            )
    }
}

impl<R, T: MemorySize> Collected<R, T> {
    fn project<U: MemorySize>(
        mut self,
        map: impl FnOnce(&AggregateOutcome, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let allowance = self.reader.decoded_bytes();
        Ok(self.reader.project(&self.outcome, allowance, map)?)
    }
}

/* === Public Functions === */
pub(crate) async fn evaluate(
    storage: &super::executions::ExecutionStorage,
    query: &AggregateQuery,
) -> Result<QueryValue<AggregateOutcome>> {
    native_aggregate(&storage.storage, query).await
}

pub(crate) async fn evaluate_projected<U: MemorySize>(
    storage: &super::executions::ExecutionStorage,
    query: &AggregateQuery,
    map: impl FnOnce(&AggregateOutcome, NonZeroUsize) -> harmos::Result<U>,
) -> Result<QueryValue<U>> {
    native_aggregate_projected(&storage.storage, query, map).await
}

pub(crate) async fn native_aggregate(
    storage: &Manager,
    query: &AggregateQuery,
) -> Result<QueryValue<AggregateOutcome>> {
    groups::validate(query)?;
    match query.source {
        AggregateSource::Cases => Ok(
            fold(storage.query(ObjectQuery::<Case>::new()).await?, query)
                .await?
                .outcome,
        ),
        AggregateSource::Cycles => Ok(fold(
            storage.query(ObjectQuery::<Cycle>::new()).await?,
            query,
        )
        .await?
        .outcome),
        AggregateSource::Executions | AggregateSource::Steps => Ok(fold(
            storage.query(ObjectQuery::<Execution>::new()).await?,
            query,
        )
        .await?
        .outcome),
        AggregateSource::Records => {
            let selected = RecordQuery::new(records::schema(), pages::decode_record);
            Ok(fold(storage.query(selected).await?, query).await?.outcome)
        }
    }
}

pub(crate) async fn native_aggregate_projected<U: MemorySize>(
    storage: &Manager,
    query: &AggregateQuery,
    map: impl FnOnce(&AggregateOutcome, NonZeroUsize) -> harmos::Result<U>,
) -> Result<QueryValue<U>> {
    groups::validate(query)?;
    match query.source {
        AggregateSource::Cases => fold(storage.query(ObjectQuery::<Case>::new()).await?, query)
            .await?
            .project(map),
        AggregateSource::Cycles => fold(storage.query(ObjectQuery::<Cycle>::new()).await?, query)
            .await?
            .project(map),
        AggregateSource::Executions | AggregateSource::Steps => {
            fold(storage.query(ObjectQuery::<Execution>::new()).await?, query)
                .await?
                .project(map)
        }
        AggregateSource::Records => {
            let selected = RecordQuery::new(records::schema(), pages::decode_record);
            fold(storage.query(selected).await?, query)
                .await?
                .project(map)
        }
    }
}

/* === Private Functions === */
async fn fold<R, T: MemorySize + pages::Projection>(
    mut reader: QueryResult<R, T>,
    query: &AggregateQuery,
) -> Result<Collected<R, T>> {
    // Use This Reader's Frozen Profile; Configured Smaller Budgets Stay Authoritative.
    let allowance = reader.decoded_bytes();
    reader.retain_view()?;
    let mut selected: Option<QueryValue<Groups>> = None;
    loop {
        let current = selected.as_deref();
        let next = reader
            .next_page_projected_including_empty(allowance, |entries, allowance| {
                pages::project(entries, current, query, allowance).map_err(pages::projection_error)
            })
            .await?;
        let Some(next) = next else {
            break;
        };
        selected = Some(next);
    }
    let selected = selected.ok_or_else(|| {
        Error::validation(
            "aggregate_query_invalid",
            "native aggregate source supplied no physical page",
            Vec::new(),
        )
    })?;
    let outcome = reader
        .project(&selected, allowance, |groups, allowance| {
            groups
                .outcome(query, allowance)
                .map_err(pages::projection_error)
        })
        .map_err(Error::from)?;
    Ok(Collected { reader, outcome })
}

/* === Tests === */

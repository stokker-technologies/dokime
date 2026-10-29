/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod fields;
mod groups;

use harmos::storage::{
    ObjectRead, RecordRead,
    query::{ObjectVersions, RecordSelection},
};

use crate::runtime::Result;
use crate::runtime::query::aggregate::{AggregateOutcome, AggregateQuery, AggregateSource};
use crate::storage::{
    Manager,
    objects::{case::Case, cycle::Cycle, execution::Execution},
    records::{self, RecordRow},
};
use fields::Row;
use groups::Groups;

pub(crate) async fn evaluate(storage: &super::executions::ExecutionStorage, query: &AggregateQuery) -> Result<AggregateOutcome> {
    native_aggregate(&storage.storage, query).await
}

pub(crate) async fn native_aggregate(storage: &Manager, query: &AggregateQuery) -> Result<AggregateOutcome> {
    groups::validate(query)?;
    let groups = match query.source {
        AggregateSource::Cases => {
            let cases = latest::<Case, _>(storage, TryInto::try_into).await?;
            let rows = cases.iter().flat_map(|page| page.iter()).map(Row::Case);
            Groups::fold(query, rows)?
        }
        AggregateSource::Cycles => {
            let cycles = latest::<Cycle, _>(storage, TryInto::try_into).await?;
            let rows = cycles.iter().flat_map(|page| page.iter()).map(Row::Cycle);
            Groups::fold(query, rows)?
        }
        AggregateSource::Executions | AggregateSource::Steps => {
            let executions = latest::<Execution, _>(storage, Execution::into_execution).await?;
            let executions = executions.iter().flat_map(|page| page.iter());
            if query.source == AggregateSource::Executions {
                Groups::fold(query, executions.map(Row::Execution))?
            } else {
                let rows = executions.flat_map(|execution| {
                    execution.steps.iter().enumerate().map(move |(ordinal, step)| Row::Step { execution, step, ordinal })
                });
                Groups::fold(query, rows)?
            }
        }
        AggregateSource::Records => {
            let entries = storage
                .read_records::<RecordRow>(RecordRead {
                    at: storage.cursor_saved(),
                    selection: RecordSelection {
                        schema: records::schema(),
                        scope: None,
                        item: None,
                        source_start: None,
                        source_end: None,
                    },
                    after: 0,
                })
                .await?
                .collect()
                .await?;
            let entries = crate::storage::projection::pages(entries, |row| row.payload.entry())?;
            let rows = entries.iter().flat_map(|page| page.iter()).map(Row::Record);
            Groups::fold(query, rows)?
        }
    };
    groups.outcome(query)
}

async fn latest<T, U>(storage: &Manager, map: impl Fn(T) -> crate::storage::Result<U>) -> Result<crate::storage::projection::Pages<U>>
where
    T: harmos::storage::StoredObject,
    crate::storage::objects::Objects: harmos::catalog::Contains<T>,
{
    let rows = storage
        .read_objects::<T>(ObjectRead {
            at: storage.cursor_saved(),
            key: None,
            versions: ObjectVersions::Latest,
        })
        .await?
        .collect()
        .await?;
    Ok(crate::storage::projection::pages(rows, |row| map(row.value))?)
}

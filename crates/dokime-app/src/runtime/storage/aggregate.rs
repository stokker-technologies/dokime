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

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::storage::{
    StoredRecord,
    query::{ObjectVersions, Query, RecordSelection, RowSlice, Source},
};

// Local Imports
use crate::Result;
use crate::runtime::query::aggregate::{AggregateOutcome, AggregateQuery, AggregateSource};
use dokime::storage::{
    Manager,
    objects::{case::Case, cycle::Cycle, execution::Execution},
    records::{self, RecordRow},
    rows::{self, Rows},
};
use fields::Row;
use groups::Groups;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn evaluate(
    storage: &super::executions::ExecutionStorage,
    query: &AggregateQuery,
) -> Result<AggregateOutcome> {
    native_aggregate(&storage.storage, query).await
}

/// Fold the latest saved domain objects, or every captured record, by the query.
pub(crate) async fn native_aggregate(
    storage: &Manager,
    query: &AggregateQuery,
) -> Result<AggregateOutcome> {
    groups::validate(query)?;
    let groups = match query.source {
        AggregateSource::Cases => {
            let cases = latest::<Case>(storage)
                .await?
                .into_iter()
                .map(|case| case.try_into())
                .collect::<std::result::Result<Vec<dokime::definition::TestCase>, _>>()
                .map_err(dokime::storage::Error::from)?;
            Groups::fold(query, &cases.iter().map(Row::Case).collect::<Vec<_>>())?
        }
        AggregateSource::Cycles => {
            let cycles = latest::<Cycle>(storage)
                .await?
                .iter()
                .map(Cycle::restore)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Groups::fold(query, &cycles.iter().map(Row::Cycle).collect::<Vec<_>>())?
        }
        AggregateSource::Executions | AggregateSource::Steps => {
            let executions = latest::<Execution>(storage)
                .await?
                .into_iter()
                .map(Execution::into_execution)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut selected = Vec::new();
            for execution in &executions {
                if query.source == AggregateSource::Executions {
                    selected.push(Row::Execution(execution));
                    continue;
                }
                for (ordinal, step) in execution.steps.iter().enumerate() {
                    selected.push(Row::Step {
                        execution,
                        step,
                        ordinal,
                    });
                }
            }
            Groups::fold(query, &selected)?
        }
        AggregateSource::Records => {
            let rows = Rows::<StoredRecord>::with_query(
                storage,
                Query {
                    at: storage.cursor_saved(),
                    source: Source::Records(RecordSelection {
                        schema: records::schema(),
                        scope: None,
                        item: None,
                        source_start: None,
                        source_end: None,
                    }),
                    transforms: Vec::new(),
                    slice: RowSlice::default(),
                },
                rows::graph::<RecordRow>()?,
            )
            .await?;
            let entries = rows
                .collect()
                .await?
                .iter()
                .map(|row| RecordRow::decode_entry(&row.payload))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Groups::fold(query, &entries.iter().map(Row::Record).collect::<Vec<_>>())?
        }
    };
    groups.outcome(query)
}

/* === Private Functions === */
async fn latest<T>(storage: &Manager) -> Result<Vec<T>>
where
    T: harmos::storage::StoredObject + harmos::schema::Described,
{
    Ok(
        rows::objects::<T>(storage, None, ObjectVersions::Latest, Vec::new())
            .await?
            .into_iter()
            .map(|revision| revision.value)
            .collect(),
    )
}

/* === Tests === */

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
    ObjectRead, RecordRead,
    query::{ObjectVersions, RecordSelection},
};

// Local Imports
use crate::runtime::Result;
use crate::runtime::query::aggregate::{AggregateOutcome, AggregateQuery, AggregateSource};
use crate::storage::{
    Manager,
    objects::{case::Case, cycle::Cycle, execution::Execution},
    records::{self, RecordRow},
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

pub(crate) async fn evaluate(storage: &super::executions::ExecutionStorage, query: &AggregateQuery) -> Result<AggregateOutcome> {
    groups::validate(query)?;
    let manager = &storage.storage;
    let at = manager.cursor_saved();
    let selected = |column: &str| {
        query.filters.iter().find_map(|filter| {
            if filter.column == column
                && matches!(
                    filter.op,
                    crate::runtime::query::aggregate::AggregateOperator::Eq | crate::runtime::query::aggregate::AggregateOperator::In
                )
                && let [dokime_domain::model::values::Value::Text(value)] = filter.values.as_slice()
            {
                Some(value.clone())
            } else {
                None
            }
        })
    };
    let primary = match query.source {
        AggregateSource::Steps => selected("execution_identifier"),
        _ => selected("identifier"),
    };
    let cycle = selected("cycle_identifier");
    let case = selected("case_identifier");
    let template = selected("template_identifier");
    let suite = selected("suite_identifier");
    let scoped = cycle.is_some() || case.is_some() || template.is_some();
    let executions = if matches!(query.source, AggregateSource::Executions | AggregateSource::Steps) && primary.is_none() && scoped {
        let scope = dokime_domain::model::execution::RecordScope {
            test_cycle_id: cycle
                .as_ref()
                .map(|id| dokime_domain::model::identity::TestCycleId::from_uuid(id.clone()))
                .transpose()
                .map_err(crate::storage::Error::from)?,
            test_case_id: case
                .as_ref()
                .map(|id| dokime_domain::model::identity::CaseId::from_uuid(id.clone()))
                .transpose()
                .map_err(crate::storage::Error::from)?,
            template_id: template
                .as_ref()
                .map(|id| dokime_domain::model::identity::TemplateId::from_uuid(id.clone()))
                .transpose()
                .map_err(crate::storage::Error::from)?,
            ..Default::default()
        };
        Some(storage.local.selected_execution_ids(at.clone(), scope).await?.into_iter().map(|id| id.to_string()).collect())
    } else {
        primary.as_ref().map(|id| vec![id.clone()])
    };

    let groups = match query.source {
        AggregateSource::Cases => {
            let keys = if let Some(id) = &primary {
                Some(vec![id.clone()])
            } else if template.is_some() || suite.is_some() {
                Some(
                    storage
                        .state_manager
                        .state()
                        .read(|state| {
                            state
                                .suites
                                .cases()
                                .values()
                                .filter(|case| {
                                    template.as_deref().is_none_or(|id| case.test_case_template_id.as_str() == id)
                                        && suite.as_deref().is_none_or(|id| case.suite_id == id)
                                })
                                .map(|case| case.test_case_id.to_string())
                                .collect()
                        })
                        .await?
                        .value,
                )
            } else {
                None
            };
            let cases = latest::<Case, _>(manager, keys, TryInto::try_into).await?;
            let rows = cases.iter().flat_map(|page| page.iter()).map(Row::Case);
            Groups::fold(query, rows)?
        }
        AggregateSource::Cycles => {
            let cycles = latest::<Cycle, _>(manager, primary.clone().map(|id| vec![id]), TryInto::try_into).await?;
            let rows = cycles.iter().flat_map(|page| page.iter()).map(Row::Cycle);
            Groups::fold(query, rows)?
        }
        AggregateSource::Executions | AggregateSource::Steps => {
            let executions = latest::<Execution, _>(manager, executions, Execution::into_execution).await?;
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
            let entries = if cycle.is_some() || case.is_some() || selected("execution_identifier").is_some() {
                let scope = crate::evidence_runtime::ScopeSelection {
                    test_cycle_id: cycle
                        .map(dokime_domain::model::identity::TestCycleId::from_uuid)
                        .transpose()
                        .map_err(crate::storage::Error::from)?,
                    test_execution_id: selected("execution_identifier")
                        .map(dokime_domain::model::identity::TestExecutionId::from_uuid)
                        .transpose()
                        .map_err(crate::storage::Error::from)?,
                    test_case_id: case
                        .map(dokime_domain::model::identity::CaseId::from_uuid)
                        .transpose()
                        .map_err(crate::storage::Error::from)?,
                    ..Default::default()
                };
                let mut pages = harmos::storage::ReadBatch::new(Vec::new());
                let mut after = 0;
                loop {
                    let rows = storage.local.scoped_records(at.clone(), scope.clone(), None, after, 0, u64::MAX).await?;
                    let Some(last) = rows.last() else { break };
                    after = last.position;
                    let rows = harmos::storage::ReadBatch::new(rows)
                        .try_map(|rows| Ok::<_, crate::runtime::Error>(rows.into_iter().map(|row| row.record).collect()))?;
                    pages = crate::storage::projection::push(pages, rows)?;
                }
                pages
            } else {
                let entries = manager
                    .read_records::<RecordRow>(RecordRead {
                        at: at.clone(),
                        selection: RecordSelection {
                            schema: records::schema(),
                            scope: None,
                            item: primary.map(harmos::Identifier::new).transpose().map_err(crate::storage::Error::from)?,
                            source_start: None,
                            source_end: None,
                        },
                        after: 0,
                    })
                    .await?
                    .collect()
                    .await?;
                crate::storage::projection::pages(entries, |row| row.payload.entry())?
            };
            let rows = entries.iter().flat_map(|page| page.iter()).map(Row::Record);
            Groups::fold(query, rows)?
        }
    };
    groups.outcome(query)
}

/* === Private Functions === */

async fn latest<T, U>(
    storage: &Manager, keys: Option<Vec<String>>, map: impl Fn(T) -> crate::storage::Result<U>,
) -> Result<crate::storage::projection::Pages<U>>
where
    T: harmos::storage::StoredObject<Key = str>,
    crate::storage::objects::Objects: harmos::catalog::Contains<T>,
{
    let mut pages = harmos::storage::ReadBatch::new(Vec::new());
    let keys = keys.map_or_else(|| vec![None], |keys| keys.into_iter().map(Some).collect());
    for key in keys {
        let rows = storage
            .read_objects::<T>(ObjectRead {
                at: storage.cursor_saved(),
                key: key.as_deref(),
                versions: ObjectVersions::Latest,
            })
            .await?
            .collect()
            .await?;
        let selected = crate::storage::projection::pages(rows, |row| map(row.value))?;
        for page in selected.iter() {
            pages = crate::storage::projection::push(pages, page.clone())?;
        }
    }
    Ok(pages)
}

/* === Tests === */

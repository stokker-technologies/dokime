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

// Third Party Imports
use harmos::storage::{
    StoredRecord,
    query::{
        Query, RecordSelection, RowSlice, Source,
        scalar::ScalarValue,
        transform::{
            FieldPath, Transform,
            filter::{Clause, Compare, Condition, Filter, Term},
        },
    },
};

// Local Imports
use super::{context_records::ContextRecord, execution::ExecutionView};
use crate::{Error, Result};
use dokime::{
    core::{CaseId, SessionId, TestCycleId, TestExecutionId},
    records::{RecordEntry, TestRecordSet},
    storage::{records, rows::Rows},
};

/* === Constants === */
// The Transform Catalog Accepts At Most This Many Terms In One `In` Condition.
const MEMBERSHIP_VALUES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */
pub(super) type PositionedRecord = super::paging::Positioned<ContextRecord>;

/* === Structs === */
/// The records an execution captured, read at its frozen evidence cut.
pub struct RecordReader {
    rows: Rows<StoredRecord>,
    view: ExecutionView,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordReader {
    pub(super) async fn open_page(view: &ExecutionView, after: u64) -> Result<Self> {
        Self::query(view, false, after).await
    }

    pub(crate) async fn open(view: &ExecutionView, context: bool) -> Result<Self> {
        Self::query(view, context, 0).await
    }

    async fn query(view: &ExecutionView, context: bool, after: u64) -> Result<Self> {
        let through = view.evidence_through()?;
        if !view.saved().covers(through)? {
            return Err(invalid("record evidence exceeds the original saved view"));
        }
        let mut transforms = vec![Transform::Filter(execution_filter(view, context)?)];
        if after > 0 {
            transforms.push(Transform::Filter(Filter {
                clauses: vec![Clause {
                    terms: vec![term(
                        "_storage.position",
                        Condition::Compare {
                            op: Compare::Gt,
                            value: ScalarValue::Unsigned(after),
                        },
                    )],
                }],
            }));
        }
        let query = Query {
            at: through.clone(),
            source: Source::Records(RecordSelection {
                schema: records::schema(),
                scope: None,
                item: None,
                source_start: None,
                source_end: None,
            }),
            transforms,
            slice: RowSlice::default(),
        };
        let rows = Rows::with_query(
            view.storage(),
            query,
            dokime::storage::rows::graph::<records::RecordRow>()?,
        )
        .await?;
        Ok(Self {
            rows,
            view: view.clone(),
        })
    }

    pub async fn next(&mut self) -> Result<Option<Vec<ContextRecord>>> {
        Ok(self
            .next_positioned()
            .await?
            .map(|rows| rows.into_iter().map(|row| row.captured).collect()))
    }

    pub(super) async fn next_positioned(&mut self) -> Result<Option<Vec<PositionedRecord>>> {
        let Some(rows) = self.rows.next().await? else {
            return Ok(None);
        };
        let execution = self.view.execution();
        let mut selected = Vec::with_capacity(rows.len());
        for row in rows {
            let mut decoded = decode(&row)?;
            apply_execution_scope(
                &mut decoded.record,
                &execution.records,
                &execution.test_cycle_id,
                &execution.test_case_id,
                &execution.test_execution_id,
            )?;
            selected.push(PositionedRecord {
                position: row.position.sequence,
                captured: decoded,
            });
        }
        Ok(Some(selected))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn term(path: &str, condition: Condition) -> Term {
    Term {
        field: FieldPath::from(path),
        condition,
        negate: false,
    }
}

fn absent(path: &str) -> Term {
    term(path, Condition::IsNull)
}

fn execution_filter(view: &ExecutionView, context: bool) -> Result<Filter> {
    let execution = view.execution();
    let subscriptions = execution.records.subscribed_records();
    let owner = |name: &str| format!("payload.scope.{name}");
    let equal = |path: String, value: &str| {
        term(
            &path,
            Condition::Compare {
                op: Compare::Eq,
                value: ScalarValue::Text(value.to_owned()),
            },
        )
    };
    let mut clauses = vec![Clause {
        terms: vec![equal(
            owner("test_execution_id"),
            execution.test_execution_id.as_str(),
        )],
    }];
    if context {
        clauses.push(Clause {
            terms: vec![
                absent(&owner("test_execution_id")),
                equal(owner("test_cycle_id"), execution.test_cycle_id.as_str()),
            ],
        });
        clauses.push(Clause {
            terms: [
                "test_execution_id",
                "test_cycle_id",
                "test_case_id",
                "step_id",
                "criterion_id",
            ]
            .into_iter()
            .map(|name| absent(&owner(name)))
            .collect(),
        });
    }
    let ids = subscriptions.keys().collect::<Vec<_>>();
    for chunk in ids.chunks(MEMBERSHIP_VALUES) {
        clauses.push(Clause {
            terms: vec![term(
                "_storage.item",
                Condition::In {
                    values: chunk
                        .iter()
                        .map(|id| ScalarValue::Text(id.as_str().to_owned()))
                        .collect(),
                },
            )],
        });
    }
    Ok(Filter { clauses })
}

fn decode(row: &StoredRecord) -> Result<ContextRecord> {
    let epoch = row
        .scope
        .as_ref()
        .ok_or_else(|| invalid("captured record has no process epoch"))?;
    if epoch.as_str().bytes().any(|byte| byte.is_ascii_uppercase())
        || row.source.is_some()
        || row.source_time.is_some()
    {
        return Err(invalid(
            "captured record envelope differs from its declared schema",
        ));
    }
    let epoch = SessionId::from_uuid(epoch.as_str()).map_err(dokime::storage::Error::from)?;
    let record = records::RecordRow::decode_entry(&row.payload)?;
    if row.item.as_str() != record.record_id.as_str() {
        return Err(invalid("captured record key differs from payload identity"));
    }
    Ok(ContextRecord { epoch, record })
}

fn apply_execution_scope(
    record: &mut RecordEntry,
    captures: &TestRecordSet,
    cycle: &TestCycleId,
    case: &CaseId,
    execution: &TestExecutionId,
) -> Result<()> {
    if let Some(scope) = captures.subscribed_records().get(&record.record_id) {
        if scope.test_execution_id.as_ref() != Some(execution)
            || scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle)
            || scope.test_case_id.as_ref().is_some_and(|id| id != case)
        {
            return Err(invalid(
                "frozen record subscription belongs to another execution",
            ));
        }
        record.scope = Some(scope.clone());
    }
    let Some(scope) = record.scope.as_ref() else {
        return Ok(());
    };
    if scope.test_execution_id.as_ref() != Some(execution) {
        return Ok(());
    }
    if scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle)
        || scope.test_case_id.as_ref().is_some_and(|id| id != case)
    {
        return Err(invalid("captured record belongs to another cycle or case"));
    }
    Ok(())
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */

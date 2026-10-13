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
use std::{collections::BTreeSet, num::NonZeroUsize};

// Third Party Imports
use harmos::MemorySize;
use harmos::storage::{
    RecordClause, RecordFilter,
    frame::{Compare, Expr, Predicate, ScalarField, ScalarPath},
    reduce::{ScalarKind, ScalarValue},
};
use serde::{Deserialize, Serialize};
// Local Imports
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use crate::records::RecordEntry;
use dokime::core::{EvidenceOrigin, RecordId, RecordScope, scope_visible_in};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeRecordFrame {
    pub records: Vec<RecordEntry>,
    pub total_records: usize,
    pub returned_records: usize,
    pub truncated: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for RuntimeRecordFrame {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.records
                .memory_size()
                .saturating_sub(std::mem::size_of::<Vec<RecordEntry>>()),
        )
    }
}

/* === Public Functions === */

pub(crate) fn record_frame<'a, 'b>(
    records: impl Iterator<Item = &'a RecordEntry> + Clone,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
    captured: impl Fn(&RecordEntry) -> Option<&'b RecordScope>,
    allowance: NonZeroUsize,
) -> harmos::Result<RuntimeRecordFrame> {
    let (count, memory) = records.clone().try_fold(
        (0usize, std::mem::size_of::<RuntimeRecordFrame>()),
        |(count, bytes), record| {
            let count = count.checked_add(1).ok_or_else(filter_capacity)?;
            let bytes = bytes
                .checked_add(record.memory_size())
                .and_then(|bytes| {
                    bytes.checked_add(captured(record).map_or(0, MemorySize::memory_size))
                })
                .ok_or_else(filter_capacity)?;
            Ok::<_, harmos::Error>((count, bytes))
        },
    )?;
    // Projection And Reduction May Retain Both Input And Output Vectors.
    if memory
        .saturating_mul(2)
        .saturating_add(count.saturating_mul(std::mem::size_of::<&RecordEntry>()))
        > allowance.get()
    {
        return Err(harmos::Error::ResourceExhausted {
            resource: "record view projection memory",
        });
    }
    let mut records = records
        .map(|record| {
            let mut projected = record.clone();
            if let Some(scope) = captured(record) {
                projected.scope = Some(scope.clone());
            }
            projected
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.timestamp
            .cmp(&right.timestamp)
            .then_with(|| left.sequence.cmp(&right.sequence))
    });
    let records = reduce_timestamped_items(
        records,
        window,
        resolution,
        |record| &record.timestamp,
        |record| Some(record.sequence),
    );
    let returned_records = records.returned.len();
    let mut returned = records.returned;
    returned.reverse();

    Ok(RuntimeRecordFrame {
        records: returned,
        total_records: records.total,
        returned_records,
        truncated: returned_records < records.total,
    })
}

pub(crate) fn execution_capture<'a>(
    mut executions: impl Iterator<Item = &'a crate::execution::TestExecution>,
    execution_id: Option<&dokime::core::TestExecutionId>,
) -> Option<&'a dokime::records::TestRecordSet> {
    let execution_id = execution_id?;
    executions
        .find(|execution| &execution.test_execution_id == execution_id)
        .map(|execution| &execution.records)
}

pub(crate) fn view_filter(
    snapshot: &crate::queries::RuntimeStateSnapshot,
    scope: Option<&RecordScope>,
    captured: Option<&dokime::records::TestRecordSet>,
) -> harmos::Result<RecordFilter> {
    let mut visible = BTreeSet::new();
    let ids = snapshot
        .state()
        .records
        .record_visibility
        .iter()
        .map(|record| &record.record_id)
        .chain(
            snapshot
                .state()
                .executions
                .values()
                .flat_map(|execution| execution.records.subscribed_records().keys()),
        );
    for id in ids {
        visible.insert(id);
        if visible.len() > 4096 {
            return Err(filter_capacity());
        }
    }
    let mut columns = [
        "session_id",
        "test_cycle_id",
        "test_case_id",
        "test_execution_id",
        "step_id",
        "criterion_id",
        "template_id",
        "suite_id",
    ]
    .into_iter()
    .map(|name| scalar(&["scope", name], ScalarKind::Text))
    .collect::<Vec<_>>();
    columns.push(scalar(&["scope", "origin", "phase"], ScalarKind::Unsigned));
    columns.push(scalar(&["scope", "origin", "attempt_id"], ScalarKind::Text));
    columns.push(ScalarField {
        path: ScalarPath::Item,
        kind: ScalarKind::Text,
    });

    let mut native = Vec::new();
    let selected = scope.filter(|scope| !scope.is_empty());
    if let Some(scope) = selected {
        let fields = [
            scope.session_id.as_ref().map(|id| id.as_str()),
            scope.test_cycle_id.as_ref().map(|id| id.as_str()),
            scope.test_case_id.as_ref().map(|id| id.as_str()),
            scope.test_execution_id.as_ref().map(|id| id.as_str()),
            scope.step_id.as_ref().map(|id| id.as_str()),
            scope.criterion_id.as_ref().map(|id| id.as_str()),
            scope.template_id.as_ref().map(|id| id.as_str()),
            scope.suite_id.as_ref().map(|id| id.as_str()),
        ];
        for (index, value) in fields.into_iter().enumerate() {
            if let Some(value) = value {
                native.push(equal(index, ScalarValue::Text(value.to_owned())));
            }
        }
        let phase = match &scope.origin {
            EvidenceOrigin::Default => None,
            EvidenceOrigin::Initialization => Some(1),
            EvidenceOrigin::Step { attempt_id } => {
                native.push(equal(9, ScalarValue::Text(attempt_id.as_str().to_owned())));
                Some(2)
            }
            EvidenceOrigin::Cleanup => Some(3),
        };
        if let Some(phase) = phase {
            native.push(equal(8, ScalarValue::Unsigned(phase)));
        }
    }
    let mut overrides = Vec::new();
    if selected.is_some()
        && let Some(captured) = captured
    {
        if captured.subscribed_records().len() > 64 {
            return Err(filter_capacity());
        }
        for (id, override_scope) in captured.subscribed_records() {
            // An Override Replaces Native Scope, Even When It Does Not Match.
            native.push(Predicate::Compare {
                expr: Expr::Col(10),
                compare: Compare::Ne,
                value: ScalarValue::Text(id.as_str().to_owned()),
            });
            if selected.is_none_or(|scope| scope_visible_in(Some(override_scope), scope)) {
                overrides.push(id);
            }
        }
    }
    if native.len() > 64 {
        return Err(filter_capacity());
    }
    let mut filter = RecordFilter {
        columns,
        clauses: Vec::new(),
    };
    let visible = visible.into_iter().collect::<Vec<_>>();
    for ids in visible.chunks(64) {
        let mut predicates = native.clone();
        predicates.push(membership(ids));
        filter.clauses.push(RecordClause { predicates });
    }
    let mut context = native;
    context.push(Predicate::Exists {
        expr: Expr::Col(1),
        present: false,
    });
    context.push(Predicate::Exists {
        expr: Expr::Col(3),
        present: false,
    });
    filter.clauses.push(RecordClause {
        predicates: context,
    });
    for ids in overrides.chunks(64) {
        filter.clauses.push(RecordClause {
            predicates: vec![membership(ids)],
        });
    }
    if filter
        .clauses
        .iter()
        .map(|clause| clause.predicates.len())
        .sum::<usize>()
        > 64
    {
        return Err(filter_capacity());
    }
    filter.check()?;
    Ok(filter)
}

/* === Private Functions === */
fn scalar(path: &[&str], kind: ScalarKind) -> ScalarField {
    ScalarField {
        path: ScalarPath::Payload(
            path.iter()
                .map(|name| harmos::Identifier::new(*name).expect("constant record scalar"))
                .collect(),
        ),
        kind,
    }
}
fn equal(index: usize, value: ScalarValue) -> Predicate {
    Predicate::Compare {
        expr: Expr::Col(index),
        compare: Compare::Eq,
        value,
    }
}
fn membership(ids: &[&RecordId]) -> Predicate {
    Predicate::In {
        expr: Expr::Col(10),
        values: ids
            .iter()
            .map(|id| ScalarValue::Text(id.as_str().to_owned()))
            .collect(),
    }
}
fn filter_capacity() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "record view selection predicates",
    }
}

/* === Tests === */

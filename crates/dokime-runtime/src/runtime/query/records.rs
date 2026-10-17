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
use std::collections::BTreeSet;

// Third Party Imports
use harmos::storage::query::{
    scalar::ScalarValue,
    transform::{
        FieldPath,
        filter::{Clause, Compare, Condition, Filter, Term},
    },
};
use serde::{Deserialize, Serialize};
// Local Imports
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use crate::records::RecordEntry;
use crate::core::{EvidenceOrigin, RecordId, RecordScope, scope_visible_in};

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
/* === Public Functions === */

pub(crate) fn record_frame<'a, 'b>(
    records: impl Iterator<Item = &'a RecordEntry>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
    captured: impl Fn(&RecordEntry) -> Option<&'b RecordScope>,
) -> RuntimeRecordFrame {
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

    RuntimeRecordFrame {
        records: returned,
        total_records: records.total,
        returned_records,
        truncated: returned_records < records.total,
    }
}

pub(crate) fn execution_capture<'a>(
    mut executions: impl Iterator<Item = &'a crate::execution::TestExecution>,
    execution_id: Option<&crate::core::TestExecutionId>,
) -> Option<&'a crate::records::TestRecordSet> {
    let execution_id = execution_id?;
    executions
        .find(|execution| &execution.test_execution_id == execution_id)
        .map(|execution| &execution.records)
}

pub(crate) fn view_filter(
    snapshot: &crate::queries::RuntimeStateSnapshot,
    scope: Option<&RecordScope>,
    captured: Option<&crate::records::TestRecordSet>,
) -> harmos::Result<Filter> {
    let visible = snapshot
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
        )
        .collect::<BTreeSet<_>>();
    let mut native = Vec::new();
    let selected = scope.filter(|scope| !scope.is_empty());
    if let Some(scope) = selected {
        let fields = [
            (
                "session_id",
                scope.session_id.as_ref().map(|id| id.as_str()),
            ),
            (
                "test_cycle_id",
                scope.test_cycle_id.as_ref().map(|id| id.as_str()),
            ),
            (
                "test_case_id",
                scope.test_case_id.as_ref().map(|id| id.as_str()),
            ),
            (
                "test_execution_id",
                scope.test_execution_id.as_ref().map(|id| id.as_str()),
            ),
            ("step_id", scope.step_id.as_ref().map(|id| id.as_str())),
            (
                "criterion_id",
                scope.criterion_id.as_ref().map(|id| id.as_str()),
            ),
            (
                "template_id",
                scope.template_id.as_ref().map(|id| id.as_str()),
            ),
            ("suite_id", scope.suite_id.as_ref().map(|id| id.as_str())),
        ];
        for (name, value) in fields {
            if let Some(value) = value {
                native.push(equal(
                    &format!("payload.scope.{name}"),
                    ScalarValue::Text(value.to_owned()),
                ));
            }
        }
        let phase = match &scope.origin {
            EvidenceOrigin::Default => None,
            EvidenceOrigin::Initialization => Some(1),
            EvidenceOrigin::Step { attempt_id } => {
                native.push(equal(
                    "payload.scope.origin.attempt_id",
                    ScalarValue::Text(attempt_id.as_str().to_owned()),
                ));
                Some(2)
            }
            EvidenceOrigin::Cleanup => Some(3),
        };
        if let Some(phase) = phase {
            native.push(equal(
                "payload.scope.origin.phase",
                ScalarValue::Unsigned(phase),
            ));
        }
    }
    let mut overrides = Vec::new();
    if selected.is_some()
        && let Some(captured) = captured
    {
        let ids = captured.subscribed_records().keys().collect::<Vec<_>>();
        if !ids.is_empty() {
            // An Override Replaces Native Scope, Even When It Does Not Match.
            native.push(Term {
                field: FieldPath::from("_storage.item"),
                condition: membership(&ids),
                negate: true,
            });
        }
        for (id, override_scope) in captured.subscribed_records() {
            if selected.is_none_or(|scope| scope_visible_in(Some(override_scope), scope)) {
                overrides.push(id);
            }
        }
    }
    let mut clauses = Vec::new();
    let visible = visible.into_iter().collect::<Vec<_>>();
    if !visible.is_empty() {
        let mut terms = native.clone();
        terms.push(Term {
            field: FieldPath::from("_storage.item"),
            condition: membership(&visible),
            negate: false,
        });
        clauses.push(Clause { terms });
    }
    let mut context = native;
    context.push(absent("payload.scope.test_cycle_id"));
    context.push(absent("payload.scope.test_execution_id"));
    clauses.push(Clause { terms: context });
    if !overrides.is_empty() {
        clauses.push(Clause {
            terms: vec![Term {
                field: FieldPath::from("_storage.item"),
                condition: membership(&overrides),
                negate: false,
            }],
        });
    }
    Ok(Filter { clauses })
}

/* === Private Functions === */
fn equal(path: &str, value: ScalarValue) -> Term {
    Term {
        field: FieldPath::from(path),
        condition: Condition::Compare {
            op: Compare::Eq,
            value,
        },
        negate: false,
    }
}

fn absent(path: &str) -> Term {
    Term {
        field: FieldPath::from(path),
        condition: Condition::IsNull,
        negate: false,
    }
}

fn membership(ids: &[&RecordId]) -> Condition {
    Condition::In {
        values: ids
            .iter()
            .map(|id| ScalarValue::Text(id.as_str().to_owned()))
            .collect(),
    }
}

/* === Tests === */

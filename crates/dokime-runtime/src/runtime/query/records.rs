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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use dokime_domain::model::execution::EvidenceOrigin;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::records::evidence::RecordEntry;

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
    records: impl Iterator<Item = &'a RecordEntry>, window: &RuntimeWindow, resolution: &RuntimeResolution,
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
    records.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then_with(|| left.sequence.cmp(&right.sequence)));
    let records = reduce_timestamped_items(records, window, resolution, |record| &record.timestamp, |record| Some(record.sequence));
    let returned_records = records.returned.len();
    let mut returned = records.returned;
    returned.reverse();

    RuntimeRecordFrame { records: returned, total_records: records.total, returned_records, truncated: returned_records < records.total }
}

pub(crate) fn execution_capture<'a>(
    mut executions: impl Iterator<Item = &'a dokime_domain::model::execution::result::TestExecution>,
    execution_id: Option<&dokime_domain::model::identity::TestExecutionId>,
) -> Option<&'a dokime_domain::state::records::set::TestRecordSet> {
    let execution_id = execution_id?;
    executions.find(|execution| &execution.test_execution_id == execution_id).map(|execution| &execution.records)
}

/// Selects the saved records one runtime view shows: visible or context records
/// within the selected scope, plus frozen subscriptions that override their native scope.
pub(crate) fn view_filter<'a>(
    snapshot: &'a crate::queries::RuntimeStateSnapshot, scope: Option<&'a RecordScope>,
    captured: Option<&'a dokime_domain::state::records::set::TestRecordSet>,
) -> impl Fn(&RecordEntry) -> bool + 'a {
    let visible = snapshot
        .state()
        .records
        .record_visibility
        .iter()
        .map(|record| &record.record_id)
        .chain(snapshot.state().executions.values().flat_map(|execution| execution.records.subscribed_records().keys()))
        .collect::<BTreeSet<_>>();
    let selected = scope.filter(|scope| !scope.is_empty());
    // An Override Replaces Native Scope, Even When It Does Not Match.
    let (replaced, overrides) = match (selected, captured) {
        (Some(scope), Some(captured)) => (
            captured.subscribed_records().keys().collect::<BTreeSet<_>>(),
            captured
                .subscribed_records()
                .iter()
                .filter(|(_, override_scope)| scope_visible_in(Some(override_scope), scope))
                .map(|(id, _)| id)
                .collect::<BTreeSet<_>>(),
        ),
        _ => (BTreeSet::new(), BTreeSet::new()),
    };
    move |record| {
        let native = selected.is_none_or(|scope| native_match(record.scope.as_ref(), scope)) && !replaced.contains(&record.record_id);
        let context = record.scope.as_ref().is_none_or(|scope| scope.test_cycle_id.is_none() && scope.test_execution_id.is_none());
        (native && (visible.contains(&record.record_id) || context)) || overrides.contains(&record.record_id)
    }
}

/* === Private Functions === */
// Every Selected Owner Field Equals The Record's; A Default Origin Selects Any Origin.
fn native_match(record: Option<&RecordScope>, selected: &RecordScope) -> bool {
    let Some(record) = record else {
        return false;
    };
    fn equal<T: PartialEq>(record: &Option<T>, selected: &Option<T>) -> bool {
        selected.as_ref().is_none_or(|value| record.as_ref() == Some(value))
    }
    equal(&record.session_id, &selected.session_id)
        && equal(&record.test_cycle_id, &selected.test_cycle_id)
        && equal(&record.test_case_id, &selected.test_case_id)
        && equal(&record.test_execution_id, &selected.test_execution_id)
        && equal(&record.step_id, &selected.step_id)
        && equal(&record.criterion_id, &selected.criterion_id)
        && equal(&record.template_id, &selected.template_id)
        && equal(&record.suite_id, &selected.suite_id)
        && (selected.origin == EvidenceOrigin::Default || record.origin == selected.origin)
}

/* === Tests === */

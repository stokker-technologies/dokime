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
use serde::{Deserialize, Serialize};

// Local Imports
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use dokime_domain::model::execution::EvidenceOrigin;
use dokime_domain::model::execution::RecordScope;

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

pub(crate) fn record_frame<'a>(
    records: impl Iterator<Item = &'a RecordEntry>, window: &RuntimeWindow, resolution: &RuntimeResolution,
    captured: impl Fn(&'a RecordEntry) -> Option<&'a RecordScope>,
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

    RuntimeRecordFrame {
        records: returned,
        total_records: records.total,
        returned_records,
        truncated: returned_records < records.total,
    }
}

pub(crate) fn execution_capture<'a>(
    mut executions: impl Iterator<Item = &'a dokime_domain::model::execution::result::TestExecution>,
    execution_id: Option<&dokime_domain::model::identity::TestExecutionId>,
) -> Option<&'a dokime_domain::state::records::set::TestRecordSet> {
    let execution_id = execution_id?;
    executions.find(|execution| &execution.test_execution_id == execution_id).map(|execution| &execution.records)
}

pub(crate) fn view_filter<'a>(
    snapshot: &'a crate::queries::RuntimeStateSnapshot, scope: Option<&'a RecordScope>,
    captured: Option<&'a dokime_domain::state::records::set::TestRecordSet>,
) -> impl Fn(&RecordEntry) -> bool + 'a {
    let selected = scope.filter(|scope| !scope.is_empty());
    move |record| {
        let projected = captured.and_then(|captures| captures.captured_scope(record)).or(record.scope.as_ref());
        let visible = snapshot.state().executions.values().any(|execution| execution.records.captured_scope(record).is_some())
            || projected.is_none_or(|scope| scope.test_cycle_id.as_ref().is_none_or(|id| snapshot.state().cycles.contains_key(id)));
        visible && selected.is_none_or(|scope| native_match(projected, scope))
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

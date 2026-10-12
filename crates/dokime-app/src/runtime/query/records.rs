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
use crate::records::RecordEntry;
use dokime::core::{RecordScope, scope_visible_in};

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

pub(crate) fn record_frame_with_subscriptions<'a, 'b>(
    records: impl Iterator<Item = &'a RecordEntry>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
    captured: impl Fn(&RecordEntry) -> Option<&'b RecordScope>,
) -> RuntimeRecordFrame {
    let mut records = records
        .filter(|record| {
            scope.is_none_or(|scope| {
                scope_visible_in(captured(record).or(record.scope.as_ref()), scope)
            })
        })
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
    execution_id: Option<&dokime::core::TestExecutionId>,
) -> Option<&'a dokime::records::TestRecordSet> {
    let execution_id = execution_id?;
    executions
        .find(|execution| &execution.test_execution_id == execution_id)
        .map(|execution| &execution.records)
}

/* === Private Functions === */
/* === Tests === */

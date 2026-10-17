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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use dokime_domain::model::values::Value;
use crate::records::RecordCondition;
use crate::records::RecordConditionEvaluation;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn evaluate_record_condition(
    condition: &RecordCondition,
    records: &[RecordEntry],
) -> RecordConditionEvaluation {
    for record in records.iter().rev() {
        if record.record_type != condition.record_type {
            continue;
        }
        if !matches_fields(&record.details, &condition.match_fields) {
            continue;
        }

        return RecordConditionEvaluation {
            satisfied: true,
            matched_sequence: Some(record.sequence),
            reason: None,
        };
    }

    RecordConditionEvaluation {
        satisfied: false,
        matched_sequence: None,
        reason: Some(format!(
            "no record matched condition {}",
            condition.record_type.as_str()
        )),
    }
}

/* === Private Functions === */
fn matches_fields(details: &BTreeMap<String, Value>, expected: &BTreeMap<String, Value>) -> bool {
    expected
        .iter()
        .all(|(name, value)| details.get(name) == Some(value))
}

/* === Tests === */

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

// Local Imports
use crate::telemetry::{TelemetryCondition, TelemetryConditionEvaluation, TelemetryConditionOperator, TelemetrySample};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn evaluate_telemetry_condition(condition: &TelemetryCondition, samples: &[TelemetrySample]) -> TelemetryConditionEvaluation {
    for sample in samples.iter().rev() {
        if sample.stream != condition.stream {
            continue;
        }
        let Some(value) = sample.values.get(&condition.field) else {
            continue;
        };
        if !matches_operator(value, condition) {
            continue;
        }

        return TelemetryConditionEvaluation { satisfied: true, matched_sequence: Some(sample.sequence), reason: None };
    }

    TelemetryConditionEvaluation {
        satisfied: false,
        matched_sequence: None,
        reason: Some(format!("no telemetry sample matched condition {}.{}", condition.stream.as_str(), condition.field)),
    }
}

/* === Private Functions === */
fn matches_operator(value: &Value, condition: &TelemetryCondition) -> bool {
    match condition.operator {
        TelemetryConditionOperator::Exists => true,
        TelemetryConditionOperator::Eq => condition.target.as_ref() == Some(value),
        TelemetryConditionOperator::Gt => compare_number(value, condition, |left, right| left > right),
        TelemetryConditionOperator::Gte => compare_number(value, condition, |left, right| left >= right),
        TelemetryConditionOperator::Lt => compare_number(value, condition, |left, right| left < right),
        TelemetryConditionOperator::Lte => compare_number(value, condition, |left, right| left <= right),
        TelemetryConditionOperator::Within => {
            let Some(left) = numeric(value) else {
                return false;
            };
            let Some(right) = condition.target.as_ref().and_then(numeric) else {
                return false;
            };
            let Some(tolerance) = condition.tolerance else {
                return false;
            };

            (left - right).abs() <= tolerance
        }
    }
}

fn compare_number(value: &Value, condition: &TelemetryCondition, predicate: impl FnOnce(f64, f64) -> bool) -> bool {
    let Some(left) = numeric(value) else {
        return false;
    };
    let Some(right) = condition.target.as_ref().and_then(numeric) else {
        return false;
    };

    predicate(left, right)
}

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Number(value) if value.is_finite() => Some(*value),
        _ => None,
    }
}

/* === Tests === */

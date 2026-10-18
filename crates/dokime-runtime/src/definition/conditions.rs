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
pub use dokime_domain::model::definition::conditions::RuntimeCondition;
pub use dokime_domain::model::definition::conditions::RuntimeConditionEvaluation;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::records::evaluate_record_condition;
use crate::telemetry::{TelemetrySample, evaluate_telemetry_condition};
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn evaluate_runtime_condition(
    condition: &RuntimeCondition, records: &[RecordEntry], telemetry: &[TelemetrySample],
) -> RuntimeConditionEvaluation {
    match condition {
        RuntimeCondition::Record { condition } => evaluate_record_condition(condition, records),
        RuntimeCondition::Telemetry { condition } => evaluate_telemetry_condition(condition, telemetry),
        RuntimeCondition::All { conditions } => {
            for condition in conditions {
                let evaluation = evaluate_runtime_condition(condition, records, telemetry);
                if !evaluation.satisfied {
                    return evaluation;
                }
            }
            RuntimeConditionEvaluation { satisfied: true, ..RuntimeConditionEvaluation::default() }
        }
        RuntimeCondition::Any { conditions } => {
            let mut reasons = Vec::new();
            for condition in conditions {
                let evaluation = evaluate_runtime_condition(condition, records, telemetry);
                if evaluation.satisfied {
                    return evaluation;
                }
                if let Some(reason) = evaluation.reason {
                    reasons.push(reason);
                }
            }
            RuntimeConditionEvaluation { satisfied: false, matched_sequence: None, reason: Some(reasons.join("; ")) }
        }
    }
}

/* === Private Functions === */

/* === Tests === */

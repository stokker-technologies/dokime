/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_protocol::v1::{self, execution_outcome::Attribution};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutcomeCounts {
    pub evaluated: u64,
    pub not_evaluated: u64,
    pub declared: u64,
    pub verdict: Option<v1::ResultStatus>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl OutcomeCounts {
    pub fn from_outcomes<'a>(outcomes: impl IntoIterator<Item = &'a v1::ExecutionOutcome>) -> Self {
        let mut counts = Self::default();
        for outcome in outcomes {
            match outcome.attribution.as_ref() {
                Some(Attribution::Evaluated(evaluated)) => {
                    counts.evaluated += 1;
                    let status = v1::ResultStatus::try_from(evaluated.status).unwrap_or(v1::ResultStatus::Blocked);
                    counts.verdict = Some(match counts.verdict {
                        Some(previous) if previous != v1::ResultStatus::Passed => previous,
                        _ => status,
                    });
                }
                Some(Attribution::Declared(_)) => counts.declared += 1,
                _ => counts.not_evaluated += 1,
            }
        }
        counts
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports
    use super::*;

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    /* === Private Functions === */

    /* === Tests === */

    #[test]
    fn cycle_outcomes_keep_declared_and_unevaluated_separate() {
        let declared = v1::ExecutionOutcome {
            attribution: Some(Attribution::Declared(v1::DeclaredOutcome {
                status: v1::ResultStatus::Passed as i32,
                importer: "thermal.importer".into(),
                reason: String::new(),
            })),
        };
        let unevaluated = v1::ExecutionOutcome {
            attribution: Some(Attribution::NotEvaluated(v1::UnevaluatedOutcome {
                termination: v1::Termination::Cancelled as i32,
                reason: String::new(),
            })),
        };
        let passed = v1::ExecutionOutcome {
            attribution: Some(Attribution::Evaluated(v1::EvaluatedOutcome {
                status: v1::ResultStatus::Passed as i32,
                reason: String::new(),
            })),
        };
        let failed = v1::ExecutionOutcome {
            attribution: Some(Attribution::Evaluated(v1::EvaluatedOutcome {
                status: v1::ResultStatus::Failed as i32,
                reason: String::new(),
            })),
        };
        assert_eq!(OutcomeCounts::from_outcomes([]), OutcomeCounts::default());
        assert_eq!(
            OutcomeCounts::from_outcomes([&declared, &unevaluated]),
            OutcomeCounts {
                declared: 1,
                not_evaluated: 1,
                evaluated: 0,
                verdict: None
            },
        );
        assert_eq!(
            OutcomeCounts::from_outcomes([&declared, &passed, &unevaluated, &failed]),
            OutcomeCounts {
                declared: 1,
                not_evaluated: 1,
                evaluated: 2,
                verdict: Some(v1::ResultStatus::Failed)
            },
        );
    }
}

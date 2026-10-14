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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::records::{RecordEntry, RecordFilter, RecordSeverity};
use dokime::core::{
    CaseId, Identifier, RecordScope, TemplateId, TestCycleId, TestExecutionId, Timestamp,
};
use dokime::telemetry::{TelemetryFilter, TelemetrySample};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordSelection {
    pub after_sequence: Option<u64>,
    pub limit: Option<usize>,
    pub record_types: Vec<Identifier>,
    pub record_type_prefixes: Vec<String>,
    pub sources: Vec<String>,
    pub severities: Vec<RecordSeverity>,
    pub min_severity: Option<RecordSeverity>,
    pub scope: ScopeSelection,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetrySelection {
    #[serde(default)]
    pub after_stream_cursors: BTreeMap<String, u64>,
    pub limit: Option<usize>,
    pub streams: Vec<Identifier>,
    pub schema_versions: Vec<u32>,
    pub sources: Vec<String>,
    pub signals: Vec<String>,
    pub scope: ScopeSelection,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    #[serde(default)]
    pub latest: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeSelection {
    pub test_cycle_id: Option<TestCycleId>,
    pub test_execution_id: Option<TestExecutionId>,
    pub test_case_id: Option<CaseId>,
    pub template_id: Option<TemplateId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordSelection {
    pub fn matches(&self, record: &RecordEntry) -> bool {
        if !self.record_types.is_empty() && !self.record_types.contains(&record.record_type) {
            return false;
        }
        if !self.record_type_prefixes.is_empty()
            && !self
                .record_type_prefixes
                .iter()
                .any(|prefix| record.record_type.as_str().starts_with(prefix))
        {
            return false;
        }
        if !self.sources.is_empty() && !self.sources.contains(&record.source) {
            return false;
        }
        if !self.severities.is_empty() && !self.severities.contains(&record.severity) {
            return false;
        }
        if let Some(minimum) = self.min_severity
            && record.severity < minimum
        {
            return false;
        }
        self.scope.matches(record.scope.as_ref())
    }

    pub fn apply(&self, records: &[RecordEntry]) -> Vec<RecordEntry> {
        self.apply_with_subscriptions(records, |_| None)
    }

    pub(crate) fn apply_with_subscriptions<'a>(
        &self,
        records: &[RecordEntry],
        captured: impl Fn(&RecordEntry) -> Option<&'a RecordScope>,
    ) -> Vec<RecordEntry> {
        let mut unscoped = self.clone();
        unscoped.scope = ScopeSelection::default();
        let window = RecordFilter {
            start: self.start.clone(),
            end: self.end.clone(),
            ..RecordFilter::default()
        };

        let mut matches = records
            .iter()
            .filter(|record| {
                self.after_sequence
                    .is_none_or(|sequence| record.sequence > sequence)
            })
            .filter(|record| {
                unscoped.matches(record)
                    && self
                        .scope
                        .matches(captured(record).or(record.scope.as_ref()))
            })
            .filter(|record| window.matches(record))
            .map(|record| {
                let mut projected = record.clone();
                if let Some(scope) = captured(record) {
                    projected.scope = Some(scope.clone());
                }
                projected
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            right
                .timestamp
                .cmp(&left.timestamp)
                .then_with(|| right.sequence.cmp(&left.sequence))
        });
        matches.truncate(self.limit.unwrap_or(usize::MAX));
        matches
    }
}

impl TelemetrySelection {
    pub fn apply(&self, samples: &[TelemetrySample]) -> Vec<TelemetrySample> {
        let mut matches = samples
            .iter()
            .filter(|sample| {
                TelemetryFilter {
                    streams: self.streams.clone(),
                    schema_versions: self.schema_versions.clone(),
                    sources: self.sources.clone(),
                    scope: None,
                    start: self.start.clone(),
                    end: self.end.clone(),
                    signals: self.signals.clone(),
                }
                .matches(sample)
            })
            .filter(|sample| self.scope.matches(sample.scope.as_ref()))
            .collect::<Vec<_>>();

        if self.latest
            && let Some(limit) = self.limit
            && matches.len() > limit
        {
            matches.drain(0..matches.len() - limit);
        }

        matches
            .into_iter()
            .take(self.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }
}

impl ScopeSelection {
    pub fn matches(&self, scope: Option<&RecordScope>) -> bool {
        let Some(scope) = scope else {
            return self.test_cycle_id.is_none()
                && self.test_execution_id.is_none()
                && self.test_case_id.is_none()
                && self.template_id.is_none();
        };

        self.test_cycle_id
            .as_ref()
            .is_none_or(|value| scope.test_cycle_id.as_ref() == Some(value))
            && self
                .test_execution_id
                .as_ref()
                .is_none_or(|value| scope.test_execution_id.as_ref() == Some(value))
            && self
                .test_case_id
                .as_ref()
                .is_none_or(|value| scope.test_case_id.as_ref() == Some(value))
            && self
                .template_id
                .as_ref()
                .is_none_or(|value| scope.template_id.as_ref() == Some(value))
    }
}

/* === Public Functions === */
pub(crate) fn scopes_correlate(left: Option<&RecordScope>, right: Option<&RecordScope>) -> bool {
    let empty = RecordScope::default();
    let left = left.unwrap_or(&empty);
    let right = right.unwrap_or(&empty);
    if left.test_execution_id != right.test_execution_id {
        return false;
    }
    if left.test_execution_id.is_none() {
        return left == right;
    }

    let attempts_agree = match (&left.origin, &right.origin) {
        (
            dokime::core::EvidenceOrigin::Step { attempt_id: left },
            dokime::core::EvidenceOrigin::Step { attempt_id: right },
        ) => left == right,
        _ => true,
    };
    attempts_agree
        && fields_agree(&left.session_id, &right.session_id)
        && fields_agree(&left.test_cycle_id, &right.test_cycle_id)
        && fields_agree(&left.test_case_id, &right.test_case_id)
        && fields_agree(&left.template_id, &right.template_id)
        && fields_agree(&left.suite_id, &right.suite_id)
        && fields_agree(&left.step_id, &right.step_id)
        && fields_agree(&left.criterion_id, &right.criterion_id)
}

/* === Private Functions === */
fn fields_agree<T: PartialEq>(left: &Option<T>, right: &Option<T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

/* === Tests === */

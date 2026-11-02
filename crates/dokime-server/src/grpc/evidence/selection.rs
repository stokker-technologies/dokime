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
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use dokime::domain::model::identity::IdentifierPattern;
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::records::evidence::RecordEntry;

/* === Constants === */

const SERVER_EVIDENCE_SELECTION_ITEMS: u64 = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Selection {
    value: dokime::domain::model::evidence::wire::CapturedRecordSelection,
    record_types: Vec<IdentifierPattern>,
    pub(super) stream: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Selection {
    pub(super) fn owners(&self) -> dokime::domains::evidence::ScopeSelection {
        let scope = self.value.scope.as_ref();
        dokime::domains::evidence::ScopeSelection {
            test_execution_id: scope
                .and_then(|scope| scope.execution_id.as_ref())
                .map(|id| dokime::TestExecutionId::from_uuid(id).expect("selection UUID was validated")),
            test_cycle_id: scope
                .and_then(|scope| scope.cycle_id.as_ref())
                .map(|id| dokime::TestCycleId::from_uuid(id).expect("selection UUID was validated")),
            ..Default::default()
        }
    }

    pub(super) fn new(value: Option<dokime::domain::model::evidence::wire::CapturedRecordSelection>) -> Result<Self, Status> {
        let mut value = value.unwrap_or_default();
        for items in [&mut value.record_types, &mut value.sources] {
            if items.len() as u64 > SERVER_EVIDENCE_SELECTION_ITEMS {
                return Err(Status::invalid_argument("selection exceeds configured alternatives"));
            }
            items.sort();
            items.dedup();
        }
        let record_types = value
            .record_types
            .iter()
            .map(|kind| IdentifierPattern::new(kind).map_err(|_| Status::invalid_argument("invalid record type pattern")))
            .collect::<Result<_, _>>()?;
        if value.severities.iter().any(|severity| {
            !matches!(*severity, common::Severity::Debug | common::Severity::Info | common::Severity::Warn | common::Severity::Error)
        }) {
            return Err(Status::invalid_argument("expected Debug, Info, Warn or Error severity"));
        }
        value.severities.sort_by_key(|severity| match severity {
            common::Severity::Debug => 2,
            common::Severity::Info => 3,
            common::Severity::Warn => 4,
            common::Severity::Error => 5,
            _ => 0,
        });
        value.severities.dedup();
        if let Some(scope) = &mut value.scope {
            for id in [
                &mut scope.session_id,
                &mut scope.cycle_id,
                &mut scope.case_id,
                &mut scope.execution_id,
                &mut scope.step_id,
                &mut scope.criterion_id,
                &mut scope.template_id,
                &mut scope.suite_id,
            ]
            .into_iter()
            .flatten()
            {
                let parsed = uuid::Uuid::parse_str(id).map_err(|_| Status::invalid_argument("invalid scope UUID"))?;
                if parsed.is_nil() {
                    return Err(Status::invalid_argument("scope UUID must not be nil"));
                }
                *id = parsed.to_string();
            }
            if scope == &common::EvidenceScope::default() {
                value.scope = None;
            }
        }
        let stream = format!(
            "evidence.records:{}",
            Sha256::digest(dokime::harmos::Message::encode(&value)).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
        );
        Ok(Self { value, record_types, stream })
    }

    pub(super) fn matches(&self, record: &RecordEntry) -> bool {
        let value = &self.value;
        if !self.record_types.is_empty() && !self.record_types.iter().any(|pattern| pattern.matches(record.record_type.as_str()))
            || !value.sources.is_empty() && !value.sources.contains(&record.source)
            || !value.severities.is_empty() && !value.severities.contains(&super::canonical::severity(record.severity))
        {
            return false;
        }
        let Some(selected) = &value.scope else {
            return true;
        };
        let matches = |scope: &dokime::RecordScope| {
            let Ok(actual) = super::super::value::canonical::scope(scope) else { return false };
            [
                (&selected.session_id, &actual.session_id),
                (&selected.cycle_id, &actual.cycle_id),
                (&selected.case_id, &actual.case_id),
                (&selected.execution_id, &actual.execution_id),
                (&selected.step_id, &actual.step_id),
                (&selected.criterion_id, &actual.criterion_id),
                (&selected.template_id, &actual.template_id),
                (&selected.suite_id, &actual.suite_id),
            ]
            .into_iter()
            .all(|(selected, actual)| selected.as_ref().is_none_or(|v| actual.as_ref() == Some(v)))
        };
        record.scope.as_ref().is_some_and(matches)
            || (selected.execution_id.is_some() || selected.cycle_id.is_some()) && record.captured_scopes.iter().any(matches)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::domain::model::{
        execution::RecordScope,
        identity::{RecordId, TestCycleId, TestExecutionId},
        records::evidence::Record,
        time::Timestamp,
    };
    use dokime::records::RecordSeverity;

    #[test]
    fn capture_membership() {
        let a = TestExecutionId::from_seed("capture-a");
        let b = TestExecutionId::from_seed("capture-b");
        let cycle = TestCycleId::from_seed("capture-cycle");
        let record = Record::new(
            RecordId::from_seed("lost-target"),
            Timestamp::new("2026-10-30T10:00:00Z"),
            dokime::domain::model::identity::Identifier::new("example.target.lost").unwrap(),
            1,
            "example",
            RecordSeverity::Info,
        );
        let mut entry = RecordEntry::from_record(1, record);
        let producer = entry.scope.clone();
        entry.captured_scopes.push(RecordScope {
            test_cycle_id: Some(cycle.clone()),
            test_execution_id: Some(a.clone()),
            ..Default::default()
        });
        let selection = |execution: Option<&TestExecutionId>, cycle: Option<&TestCycleId>| {
            Selection::new(Some(dokime::domain::model::evidence::wire::CapturedRecordSelection {
                record_types: vec!["example.target.lost".into()],
                scope: Some(common::EvidenceScope {
                    execution_id: execution.map(ToString::to_string),
                    cycle_id: cycle.map(ToString::to_string),
                    ..Default::default()
                }),
                ..Default::default()
            }))
            .unwrap()
        };
        assert!(selection(Some(&a), None).matches(&entry));
        assert!(selection(None, Some(&cycle)).matches(&entry));
        assert!(!selection(Some(&b), None).matches(&entry));
        entry.captured_scopes.push(RecordScope {
            test_execution_id: Some(b.clone()),
            ..Default::default()
        });
        assert!(selection(Some(&b), None).matches(&entry));
        assert_eq!(entry.scope, producer);
        entry.captured_scopes.clear();
        entry.scope = Some(RecordScope {
            test_execution_id: Some(a.clone()),
            test_cycle_id: Some(cycle.clone()),
            ..Default::default()
        });
        assert!(selection(Some(&a), None).matches(&entry));
        assert!(selection(None, Some(&cycle)).matches(&entry));
        assert!(!selection(Some(&b), None).matches(&entry));
    }
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Captured Evidence Selection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result};
use dokime_storage::{Evidence, ExecutionCut};
use prost::Message;

use dokime_protocol::host::evidence::CapturedRecord;
use dokime_protocol::host::evidence::CapturedRecordSelection;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::Severity;
use dokime_protocol::model::Suites;
use dokime_protocol::model::evidence_origin_kind::Kind as EvidenceOriginKindKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use harmos_protocol::model::Declarations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn validate(selection: &CapturedRecordSelection) -> Result<()> {
    if selection.severities.iter().any(|severity| {
        !matches!(Severity::try_from(*severity), Ok(Severity::Debug | Severity::Info | Severity::Warning | Severity::Failure))
    }) {
        return Err(Error::Invalid("List Records: Select Debug, Info, Warning or Error Severity".into()));
    }
    if let Some(scope) = &selection.scope
        && (scope.session_id.is_some() || scope.case_id.is_some())
    {
        return Err(Error::Invalid("Read Evidence: Select an Execution or Cycle Instead of a Retired Session or Planned Case".into()));
    }
    if selection
        .scope
        .as_ref()
        .and_then(|scope| scope.origin.as_ref())
        .and_then(|origin| origin.kind.as_ref())
        .and_then(|kind| kind.kind.as_ref())
        .is_some_and(|kind| matches!(kind, EvidenceOriginKindKind::StepAttemptId(_)))
    {
        return Err(Error::Invalid("Read Evidence: Select Execution and Step Scopes Instead of a Retired Opaque Attempt Identity".into()));
    }
    Ok(())
}
pub(crate) fn matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return value.strip_prefix(prefix).is_some_and(|tail| tail.starts_with('.'));
    }
    value == pattern
}
pub(crate) fn scope_matches(selected: Option<&EvidenceScope>, cut: &ExecutionCut, evidence: &Evidence) -> bool {
    let Some(scope) = selected else {
        return true;
    };
    if scope.execution_id.as_ref().is_some_and(|id| id != &cut.execution.identifier)
        || scope.cycle_id.as_ref().is_some_and(|id| id != &cut.execution.cycle)
        || scope.step_id.as_ref().is_some_and(|id| id != &evidence.step)
    {
        return false;
    }
    let run = cut.execution.origin.as_ref().and_then(|origin| match origin.kind.as_ref() {
        Some(OriginKind::PluginRun(run)) => Some(run),
        _ => None,
    });
    if scope.criterion_id.as_ref().is_some_and(|id| {
        run.and_then(|run| run.test_case.as_ref()).is_none_or(|case| {
            !case.criteria.iter().any(|criterion| {
                criterion.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == id) && criterion.operation == evidence.operation
            })
        })
    }) {
        return false;
    }
    if scope.template_id.as_ref().is_some_and(|id| run.is_none_or(|run| id != &run.template)) {
        return false;
    }
    if let Some(suite) = &scope.suite_id {
        let Ok(catalog) = Declarations::decode(cut.execution.catalog.as_slice()) else {
            return false;
        };
        let found = catalog
            .sections
            .iter()
            .filter(|section| section.r#type == "dokime.model.v1.Suites")
            .filter_map(|section| Suites::decode(section.payload.as_slice()).ok())
            .flat_map(|suites| suites.suites)
            .any(|item| {
                item.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == suite)
                    && run.is_some_and(|run| {
                        item.templates
                            .iter()
                            .any(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template))
                    })
            });
        if !found {
            return false;
        }
    }
    if let Some(origin) = scope.origin.as_ref().and_then(|origin| origin.kind.as_ref()).and_then(|kind| kind.kind.as_ref()) {
        match origin {
            EvidenceOriginKindKind::Initialization(_) if !evidence.operation.ends_with(".initialize") => {
                return false;
            }
            EvidenceOriginKindKind::Cleanup(_) if !evidence.operation.ends_with(".cleanup") => return false,
            EvidenceOriginKindKind::StepAttemptId(_) => return false,
            _ => (),
        }
    }
    true
}
pub(crate) fn project(cut: &ExecutionCut, evidence: &Evidence) -> Result<Option<CapturedRecord>> {
    let (kind, payload, timestamp, severity, message) = if let Some(kind) = evidence.kind.strip_prefix("record:") {
        let record = RecordsAppendRequest::decode(evidence.payload.as_slice())?;
        (kind.into(), record.payload, record.source_at, Severity::Info as i32, None)
    } else if evidence.kind == "log" {
        let log = LogsAppendRequest::decode(evidence.payload.as_slice())?;
        ("dokime.log".into(), evidence.payload.clone(), log.at, log.severity, Some(log.message))
    } else {
        return Ok(None);
    };
    let source = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(OriginKind::PluginRun(run)) => run.plugin.clone(),
        _ => "import".into(),
    };
    Ok(Some(CapturedRecord {
        record_id: format!("{}:{}", cut.execution.identifier, evidence.position),
        sequence: evidence.position,
        timestamp: timestamp.or_else(|| Some(crate::host::projection::timestamp(evidence.source_ns))),
        received_at: Some(crate::host::projection::timestamp(evidence.receipt_ns)),
        record_type: kind,
        schema_version: 1,
        payload,
        source,
        severity,
        message,
        scope: Some(EvidenceScope {
            execution_id: Some(cut.execution.identifier.clone()),
            cycle_id: Some(cut.execution.cycle.clone()),
            step_id: (!evidence.step.is_empty()).then(|| evidence.step.clone()),
            ..Default::default()
        }),
        ..Default::default()
    }))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Evidence, ExecutionCut, RecordsAppendRequest, project, scope_matches};
    use dokime_protocol::model::EvidenceScope;
    use dokime_storage::{Execution, ExecutionState};
    use prost::Message;
    use prost_types::Timestamp;

    fn cut() -> ExecutionCut {
        ExecutionCut {
            execution: Execution {
                identifier: "run".into(),
                cycle: "cycle".into(),
                ..Default::default()
            },
            state: ExecutionState::Open,
            seal: Vec::new(),
            frontier: 17,
        }
    }

    #[test]
    fn captured_records_preserve_payload_source_timestamp_and_sequence() {
        let request = RecordsAppendRequest {
            execution: "run".into(),
            kind: "reading".into(),
            payload: vec![0, 255, 1],
            source_at: Some(Timestamp { seconds: 7, nanos: 12 }),
            ..Default::default()
        };
        let evidence = Evidence {
            kind: "record:reading".into(),
            payload: request.encode_to_vec(),
            position: 17,
            receipt_ns: 9_000_000_000,
            ..Default::default()
        };
        let captured = project(&cut(), &evidence).unwrap().unwrap();
        assert_eq!(captured.payload, request.payload);
        assert_eq!(captured.timestamp, request.source_at);
        assert_eq!(captured.sequence, evidence.position);
        assert_eq!(captured.received_at, Some(Timestamp { seconds: 9, nanos: 0 }));
    }

    #[test]
    fn scope_filters_cannot_select_foreign_execution_cycle_or_step() {
        let evidence = Evidence {
            step: "measure".into(),
            ..Default::default()
        };
        let mut scope = EvidenceScope {
            execution_id: Some("run".into()),
            cycle_id: Some("cycle".into()),
            step_id: Some("measure".into()),
            ..Default::default()
        };
        assert!(scope_matches(Some(&scope), &cut(), &evidence));
        scope.execution_id = Some("foreign".into());
        assert!(!scope_matches(Some(&scope), &cut(), &evidence));
        scope.execution_id = Some("run".into());
        scope.cycle_id = Some("foreign".into());
        assert!(!scope_matches(Some(&scope), &cut(), &evidence));
        scope.cycle_id = Some("cycle".into());
        scope.step_id = Some("foreign".into());
        assert!(!scope_matches(Some(&scope), &cut(), &evidence));
    }
}

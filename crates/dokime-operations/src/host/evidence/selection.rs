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
use dokime_protocol::model;
use dokime_storage::{Evidence, ExecutionCut};
use prost::Message;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn validate(selection: &dokime_protocol::host::evidence::CapturedRecordSelection) -> Result<()> {
    if selection.severities.iter().any(|severity| {
        !matches!(
            dokime_protocol::model::Severity::try_from(*severity),
            Ok(dokime_protocol::model::Severity::Debug
                | dokime_protocol::model::Severity::Info
                | dokime_protocol::model::Severity::Warning
                | dokime_protocol::model::Severity::Failure)
        )
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
        .is_some_and(|kind| matches!(kind, dokime_protocol::model::evidence_origin_kind::Kind::StepAttemptId(_)))
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
pub(crate) fn scope_matches(selected: Option<&dokime_protocol::model::EvidenceScope>, cut: &ExecutionCut, evidence: &Evidence) -> bool {
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
        Some(dokime_protocol::model::origin::Kind::PluginRun(run)) => Some(run),
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
        let Ok(catalog) = harmos_protocol::model::Declarations::decode(cut.execution.catalog.as_slice()) else {
            return false;
        };
        let found = catalog
            .sections
            .iter()
            .filter(|section| section.r#type == "dokime.model.v1.Suites")
            .filter_map(|section| dokime_protocol::model::Suites::decode(section.payload.as_slice()).ok())
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
            dokime_protocol::model::evidence_origin_kind::Kind::Initialization(_) if !evidence.operation.ends_with(".initialize") => {
                return false;
            }
            dokime_protocol::model::evidence_origin_kind::Kind::Cleanup(_) if !evidence.operation.ends_with(".cleanup") => return false,
            dokime_protocol::model::evidence_origin_kind::Kind::StepAttemptId(_) => return false,
            _ => (),
        }
    }
    true
}
pub(crate) fn project(cut: &ExecutionCut, evidence: &Evidence) -> Result<Option<dokime_protocol::host::evidence::CapturedRecord>> {
    let (kind, payload, timestamp, severity, message) = if let Some(kind) = evidence.kind.strip_prefix("record:") {
        let record = dokime_protocol::host::evidence::RecordsAppendRequest::decode(evidence.payload.as_slice())?;
        (kind.into(), record.payload, record.source_at, dokime_protocol::model::Severity::Info as i32, None)
    } else if evidence.kind == "log" {
        let log = dokime_protocol::host::evidence::LogsAppendRequest::decode(evidence.payload.as_slice())?;
        ("dokime.log".into(), evidence.payload.clone(), log.at, log.severity, Some(log.message))
    } else {
        return Ok(None);
    };
    let source = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(dokime_protocol::model::origin::Kind::PluginRun(run)) => run.plugin.clone(),
        _ => "import".into(),
    };
    Ok(Some(dokime_protocol::host::evidence::CapturedRecord {
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
        scope: Some(dokime_protocol::model::EvidenceScope {
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

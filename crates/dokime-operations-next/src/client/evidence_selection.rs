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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use dokime_storage::{Evidence, ExecutionCut};
use prost::Message;
use sha2::{Digest, Sha256};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn key(selection: &v1::CapturedRecordSelection) -> String {
    format!("evidence.records:{}", Sha256::digest(selection.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect::<String>())
}
pub(super) fn validate(selection: &v1::CapturedRecordSelection) -> Result<()> {
    if selection.severities.iter().any(|severity| {
        !matches!(
            v1::Severity::try_from(*severity),
            Ok(v1::Severity::Debug | v1::Severity::Info | v1::Severity::Warning | v1::Severity::Error)
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
        .is_some_and(|kind| matches!(kind, v1::evidence_origin_kind::Kind::StepAttemptId(_)))
    {
        return Err(Error::Invalid("Read Evidence: Select Execution and Step Scopes Instead of a Retired Opaque Attempt Identity".into()));
    }
    Ok(())
}
pub(super) fn matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return value.strip_prefix(prefix).is_some_and(|tail| tail.starts_with('.'));
    }
    value == pattern
}
pub(super) fn scope_matches(selected: Option<&v1::EvidenceScope>, cut: &ExecutionCut, evidence: &Evidence) -> bool {
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
        Some(v1::origin::Kind::PluginRun(run)) => Some(run),
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
        let Ok(catalog) = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice()) else {
            return false;
        };
        let found = catalog
            .sections
            .iter()
            .filter(|section| section.r#type == "dokime.v1.Suites")
            .filter_map(|section| v1::Suites::decode(section.payload.as_slice()).ok())
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
            v1::evidence_origin_kind::Kind::Initialization(_) if !evidence.operation.ends_with(".initialize") => return false,
            v1::evidence_origin_kind::Kind::Cleanup(_) if !evidence.operation.ends_with(".cleanup") => return false,
            v1::evidence_origin_kind::Kind::StepAttemptId(_) => return false,
            _ => (),
        }
    }
    true
}
pub(super) fn project(cut: &ExecutionCut, evidence: &Evidence) -> Result<Option<v1::CapturedRecord>> {
    let (kind, payload, timestamp, severity, message) = if let Some(kind) = evidence.kind.strip_prefix("record:") {
        let record = v1::PublishRecord::decode(evidence.payload.as_slice())?;
        (kind.into(), record.payload, record.at, v1::Severity::Info as i32, None)
    } else if evidence.kind == "log" {
        let log = v1::WriteLog::decode(evidence.payload.as_slice())?;
        ("dokime.log".into(), evidence.payload.clone(), log.at, log.severity, Some(log.message))
    } else {
        return Ok(None);
    };
    let source = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(v1::origin::Kind::PluginRun(run)) => run.plugin.clone(),
        _ => "import".into(),
    };
    Ok(Some(v1::CapturedRecord {
        record_id: format!("{}:{}", cut.execution.identifier, evidence.position),
        sequence: evidence.position,
        timestamp: timestamp.or_else(|| Some(super::projection::timestamp(evidence.source_ns))),
        received_at: Some(super::projection::timestamp(evidence.receipt_ns)),
        record_type: kind,
        schema_version: 1,
        payload,
        source,
        severity,
        message,
        scope: Some(v1::EvidenceScope {
            execution_id: Some(cut.execution.identifier.clone()),
            cycle_id: Some(cut.execution.cycle.clone()),
            step_id: (!evidence.step.is_empty()).then(|| evidence.step.clone()),
            ..Default::default()
        }),
        ..Default::default()
    }))
}
pub(super) fn cursor(context: &Context, stream: &str, position: u64) -> v1::Cursor {
    v1::Cursor {
        epoch: context.host.identity().backend_id,
        stream: stream.into(),
        position,
    }
}
pub(super) fn position(context: &Context, stream: &str, cursor: Option<&v1::Cursor>, maximum: u64, fallback: u64) -> Result<u64> {
    let Some(cursor) = cursor else {
        return Ok(fallback);
    };
    if cursor.epoch != context.host.identity().backend_id || cursor.stream != stream || cursor.position > maximum {
        return Err(Error::Invalid(
            "Read Evidence Cursor: Resynchronize Without Cursors After Changing the Selection or Storage Root".into(),
        ));
    }
    Ok(cursor.position)
}

/* === Private Functions === */

/* === Tests === */

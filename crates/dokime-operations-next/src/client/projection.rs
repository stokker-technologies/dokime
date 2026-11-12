/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage};

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use dokime_storage::{ExecutionSnapshot, ExecutionState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn limit(requested: u32, maximum: usize) -> usize {
    if requested == 0 { maximum } else { (requested as usize).min(maximum) }
}
pub(super) fn bounded(count: usize, maximum: usize) -> Result<()> {
    if count > maximum {
        return Err(Error::Invalid("Read Exceeds the New Host's Page Bound; Narrow the Selection".into()));
    }
    Ok(())
}
pub(super) fn timestamp(at_ns: i64) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
}
pub(super) fn scope(snapshot: &ExecutionSnapshot) -> v1::EvidenceScope {
    v1::EvidenceScope {
        execution_id: Some(snapshot.execution.identifier.clone()),
        cycle_id: Some(snapshot.execution.cycle.clone()),
        ..Default::default()
    }
}
pub(super) fn execution_scope(scope: Option<&v1::EvidenceScope>) -> Result<&str> {
    let scope = scope.ok_or_else(|| Error::Invalid("Read Requires an Execution Scope".into()))?;
    if scope.origin.is_some()
        || scope.session_id.is_some()
        || scope.case_id.is_some()
        || scope.criterion_id.is_some()
        || scope.template_id.is_some()
        || scope.suite_id.is_some()
    {
        return Err(Error::Invalid("Read Scope Filters Are Not Yet Served by the New Host".into()));
    }
    scope
        .execution_id
        .as_deref()
        .filter(|identifier| !identifier.is_empty())
        .ok_or_else(|| Error::Invalid("Read Requires an Execution Identifier".into()))
}
pub(super) fn check_scope(scope: &v1::EvidenceScope, snapshot: &ExecutionSnapshot) -> Result<()> {
    if scope.cycle_id.as_ref().is_some_and(|cycle| cycle != &snapshot.execution.cycle) {
        return Err(Error::Invalid("Read Requires an Execution in the Selected Cycle".into()));
    }
    Ok(())
}
pub(super) fn prompt_definition(snapshot: &ExecutionSnapshot, question: &v1::OperatorQuestion) -> Result<v1::Prompt> {
    if let Some(definition) = &question.definition {
        return Ok(definition.clone());
    }
    let catalog = h::Catalog::decode(snapshot.execution.catalog.as_slice())?;
    for section in catalog.sections {
        if section.r#type != "dokime.v1.PromptCatalog" {
            continue;
        }
        for prompt in v1::PromptCatalog::decode(section.payload.as_slice())?.prompts {
            if prompt.metadata.as_ref().is_some_and(|metadata| metadata.identifier == question.prompt_type) {
                return Ok(prompt);
            }
        }
    }
    Err(Error::Invalid("Read Prompt Definition: Declare the Prompt in the Frozen Catalog".into()))
}
pub(super) fn summary(snapshot: &ExecutionSnapshot, controls: v1::ExecutionControls) -> v1::ExecutionSummary {
    let state = match snapshot.state {
        ExecutionState::Open => v1::ExecutionState::Open,
        ExecutionState::Closed => v1::ExecutionState::Closed,
        ExecutionState::Sealed => v1::ExecutionState::Sealed,
    };
    v1::ExecutionSummary {
        execution_id: snapshot.execution.identifier.clone(),
        state: state as i32,
        outcome: snapshot.execution.outcome.clone(),
        controls: Some(controls),
        ..Default::default()
    }
}
pub(super) fn counts<'a>(outcomes: impl IntoIterator<Item = &'a v1::ExecutionOutcome>) -> v1::ExecutionOutcomeCounts {
    let counts = dokime_storage::OutcomeCounts::from_outcomes(outcomes);
    v1::ExecutionOutcomeCounts {
        evaluated_count: counts.evaluated,
        not_evaluated_count: counts.not_evaluated,
        declared_count: counts.declared,
        result_status: counts.verdict.map(|verdict| verdict as i32),
    }
}
pub(super) fn schemas(snapshot: &ExecutionSnapshot) -> Result<v1::FrozenSchemas> {
    let catalog = h::Catalog::decode(snapshot.execution.catalog.as_slice())?;
    let mut records = Vec::new();
    for section in &catalog.sections {
        if section.r#type != "dokime.v1.RecordContracts" {
            continue;
        }
        for contract in v1::RecordContracts::decode(section.payload.as_slice())?.records {
            records.push(v1::RecordSchema {
                record_type: contract.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default(),
                schema_version: 1,
                metadata: contract.metadata,
                ..Default::default()
            });
        }
    }
    let streams = catalog
        .streams
        .iter()
        .map(|stream| v1::TelemetrySchema {
            stream: stream.id.clone(),
            schema_version: 1,
            ..Default::default()
        })
        .collect();
    Ok(v1::FrozenSchemas { catalog: Some(catalog), records, streams })
}
pub(super) fn records(snapshot: &ExecutionSnapshot, kinds: &[String], step: Option<&str>) -> Result<Vec<v1::CapturedRecord>> {
    let mut records = Vec::new();
    for evidence in &snapshot.evidence {
        let Some(kind) = evidence.kind.strip_prefix("record:") else {
            continue;
        };
        if !kinds.is_empty() && !kinds.iter().any(|selected| selected == kind) {
            continue;
        }
        if !snapshot.step_contains(step.unwrap_or_default(), evidence.position) {
            continue;
        }
        let record = v1::PublishRecord::decode(evidence.payload.as_slice())?;
        let mut scope = scope(snapshot);
        scope.step_id = Some(evidence.step.clone());
        records.push(v1::CapturedRecord {
            record_id: format!("{}:{}", snapshot.execution.identifier, evidence.position),
            sequence: evidence.position,
            timestamp: record.at,
            received_at: Some(timestamp(evidence.receipt_ns)),
            record_type: record.kind,
            payload: record.payload,
            schema_version: 1,
            scope: Some(scope),
            ..Default::default()
        });
    }
    Ok(records)
}
pub(super) fn samples(snapshot: &ExecutionSnapshot, streams: &[String], step: Option<&str>) -> Result<Vec<v1::TelemetrySample>> {
    let catalog = h::Catalog::decode(snapshot.execution.catalog.as_slice())?;
    let pool = DescriptorPool::from_file_descriptor_set(
        catalog.types.clone().ok_or_else(|| Error::Invalid("Read Samples Requires Frozen Descriptors".into()))?,
    )
    .map_err(|error| Error::Invalid(format!("Read Sample Descriptors: {error}")))?;
    let mut samples = Vec::new();
    for sample in &snapshot.samples {
        if !streams.is_empty() && !streams.contains(&sample.stream) {
            continue;
        }
        if !snapshot.step_contains(step.unwrap_or_default(), sample.position) {
            continue;
        }
        let stream = catalog
            .streams
            .iter()
            .find(|stream| stream.id == sample.stream)
            .ok_or_else(|| Error::Invalid("Read Sample Requires a Frozen Stream Contract".into()))?;
        let descriptor =
            pool.get_message_by_name(&stream.row).ok_or_else(|| Error::Invalid("Read Sample Requires a Frozen Row Descriptor".into()))?;
        let row = DynamicMessage::decode(descriptor, sample.row.as_slice())?;
        let mut values = std::collections::HashMap::new();
        for field in row.descriptor().fields() {
            values.insert(field.name().into(), value(row.get_field(&field).as_ref()));
        }
        samples.push(v1::TelemetrySample {
            sample_id: format!("{}:{}", snapshot.execution.identifier, sample.position),
            stream: sample.stream.clone(),
            sequence: sample.sequence,
            schema_version: 1,
            timestamp: Some(timestamp(sample.source_ns)),
            received_at: Some(timestamp(sample.receipt_ns)),
            scope: Some(scope(snapshot)),
            values: Some(v1::MapValue { values }),
            payload: sample.row.clone(),
            ..Default::default()
        });
    }
    Ok(samples)
}
pub(super) fn prompt_id(execution: &str, prompt: &str) -> String {
    format!("{execution}/{prompt}")
}
pub(super) fn pending(context: &Context, identifier: &str) -> Result<(String, v1::OperatorQuestion)> {
    context
        .host
        .pending()?
        .into_iter()
        .find(|(execution, question)| prompt_id(execution, &question.prompt) == identifier)
        .ok_or_else(|| Error::Invalid(format!("Read or Answer Pending Prompt {identifier}: Prompt Is Not Pending")))
}
pub(super) fn report(snapshot: &ExecutionSnapshot, counts: &v1::ExecutionOutcomeCounts) -> String {
    let identifier = escape(&snapshot.execution.identifier);
    let mut html = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>Execution {identifier}</title><h1>Execution {identifier}</h1>"
    );
    html.push_str(&format!("<p>State: {:?}</p><p>Termination: {}</p>", snapshot.state, snapshot.execution.termination));
    html.push_str(&format!(
        "<p>Outcome: {}</p><h2>Evidence</h2><table><tr><th>Position</th><th>Kind</th><th>Step</th></tr>",
        escape(&format!("{:?}", snapshot.execution.outcome))
    ));
    for evidence in &snapshot.evidence {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            evidence.position,
            escape(&evidence.kind),
            escape(&evidence.step)
        ));
    }
    html.push_str("</table><h2>Stream Samples</h2><table><tr><th>Stream</th><th>Sequence</th><th>Source Time [ns]</th></tr>");
    for sample in &snapshot.samples {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>", escape(&sample.stream), sample.sequence, sample.source_ns));
    }
    html.push_str(&format!(
        "</table><dl><dt>Evaluated</dt><dd>{}</dd><dt>Not Evaluated</dt><dd>{}</dd><dt>Declared</dt><dd>{}</dd></dl></html>",
        counts.evaluated_count, counts.not_evaluated_count, counts.declared_count
    ));
    html
}
pub(super) fn report_frame(kind: v1::report_html_frame_payload::Kind) -> v1::ReportsDownloadExecutionReportOutput {
    let item = v1::ReportHtmlFrame {
        payload: Some(v1::ReportHtmlFramePayload { kind: Some(kind) }),
    };
    v1::ReportsDownloadExecutionReportOutput {
        value: Some(v1::reports_download_execution_report_output::Value::Item(item)),
    }
}

/* === Private Functions === */

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}
pub(super) fn value(value: &prost_reflect::Value) -> v1::Value {
    use prost_reflect::Value as V;
    use v1::value::Kind;
    let kind = match value {
        V::Bool(value) => Kind::Boolean(*value),
        V::I32(value) => Kind::Integer(i64::from(*value)),
        V::I64(value) => Kind::Integer(*value),
        V::U32(value) => Kind::UnsignedInteger(u64::from(*value)),
        V::U64(value) => Kind::UnsignedInteger(*value),
        V::F32(value) => Kind::Number(f64::from(*value)),
        V::F64(value) => Kind::Number(*value),
        V::String(value) => Kind::Text(value.clone()),
        V::Bytes(value) => Kind::Data(value.to_vec()),
        V::EnumNumber(value) => Kind::Integer(i64::from(*value)),
        V::Message(value) => Kind::Data(value.encode_to_vec()),
        V::List(values) => Kind::List(v1::ValueList {
            values: values.iter().map(self::value).collect(),
        }),
        V::Map(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(key, _)| format!("{key:?}"));
            let values = entries
                .into_iter()
                .map(|(key, item)| {
                    let key = match key {
                        prost_reflect::MapKey::Bool(key) => Kind::Boolean(*key),
                        prost_reflect::MapKey::I32(key) => Kind::Integer(i64::from(*key)),
                        prost_reflect::MapKey::I64(key) => Kind::Integer(*key),
                        prost_reflect::MapKey::U32(key) => Kind::UnsignedInteger(u64::from(*key)),
                        prost_reflect::MapKey::U64(key) => Kind::UnsignedInteger(*key),
                        prost_reflect::MapKey::String(key) => Kind::Text(key.clone()),
                    };
                    v1::Value {
                        kind: Some(Kind::List(v1::ValueList {
                            values: vec![v1::Value { kind: Some(key) }, self::value(item)],
                        })),
                    }
                })
                .collect();
            Kind::List(v1::ValueList { values })
        }
    };
    v1::Value { kind: Some(kind) }
}

/* === Tests === */

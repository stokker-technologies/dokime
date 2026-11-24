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
use harmos_protocol::model as h;
use prost::Message;
use prost_reflect::{DynamicMessage, ReflectMessage};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ExecutionControls;
use dokime_protocol::host::ExecutionOutcome;
use dokime_protocol::host::ExecutionOutcomeCounts;
use dokime_protocol::host::ExecutionSummary;
use dokime_protocol::host::FrozenSchemas;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::evidence::CapturedRecord;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::TelemetrySample;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::Prompt;
use dokime_protocol::model::Value as ProductValue;
use dokime_storage::Evidence;
use dokime_storage::FrozenCatalog;
use dokime_storage::OutcomeCounts;
use dokime_storage::{ExecutionSnapshot, ExecutionState};
use prost_reflect::MapKey;
use prost_reflect::Value;
use prost_types::Timestamp;
use std::collections::HashMap;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn limit(requested: u32, maximum: usize) -> usize {
    if requested == 0 { maximum } else { (requested as usize).min(maximum) }
}

pub(crate) fn timestamp(at_ns: i64) -> Timestamp {
    Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
}
pub(crate) fn scope(snapshot: &ExecutionSnapshot) -> dokime_protocol::model::EvidenceScope {
    dokime_protocol::model::EvidenceScope {
        execution_id: Some(snapshot.execution.identifier.clone()),
        cycle_id: Some(snapshot.execution.cycle.clone()),
        ..Default::default()
    }
}
pub(crate) fn entry_scope(snapshot: &ExecutionSnapshot, entry: &Evidence) -> Result<EvidenceScope> {
    let mut scope = scope(snapshot);
    if !entry.step.is_empty() {
        scope.step_id = Some(entry.step.clone());
    }
    let Some(dokime_protocol::model::origin::Kind::PluginRun(run)) =
        snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref())
    else {
        return Ok(scope);
    };
    scope.template_id = Some(run.template.clone());
    let catalog = h::Declarations::decode(snapshot.execution.catalog.as_slice())?;
    for section in catalog.sections {
        if section.r#type != "dokime.model.v1.Suites" {
            continue;
        }
        for suite in dokime_protocol::model::Suites::decode(section.payload.as_slice())?.suites {
            for template in suite.templates {
                if template.metadata.as_ref().is_none_or(|metadata| metadata.identifier != run.template) {
                    continue;
                }
                scope.suite_id = suite.metadata.as_ref().map(|metadata| metadata.identifier.clone());
                let kind = if template.initialize.as_ref() == Some(&entry.operation) {
                    Some(dokime_protocol::model::evidence_origin_kind::Kind::Initialization(true))
                } else if template.cleanup.as_ref() == Some(&entry.operation) {
                    Some(dokime_protocol::model::evidence_origin_kind::Kind::Cleanup(true))
                } else {
                    snapshot
                        .evidence
                        .iter()
                        .rev()
                        .find(|attempt| {
                            attempt.kind == "attempt:start"
                                && attempt.position <= entry.position
                                && attempt.operation == entry.operation
                                && attempt.step == entry.step
                                && attempt.attempt == entry.attempt
                        })
                        .map(|attempt| {
                            dokime_protocol::model::evidence_origin_kind::Kind::StepAttemptId(format!(
                                "{}/{}",
                                snapshot.execution.identifier, attempt.position
                            ))
                        })
                };
                if let Some(kind) = kind {
                    scope.origin = Some(dokime_protocol::model::EvidenceOrigin {
                        kind: Some(dokime_protocol::model::EvidenceOriginKind { kind: Some(kind) }),
                    });
                }
            }
        }
    }
    if let Some(case) = &run.test_case {
        scope.criterion_id = case
            .criteria
            .iter()
            .find(|criterion| criterion.operation == entry.operation)
            .and_then(|criterion| criterion.metadata.as_ref())
            .map(|metadata| metadata.identifier.clone());
    }
    Ok(scope)
}
pub(crate) fn execution_scope(scope: Option<&dokime_protocol::model::EvidenceScope>) -> Result<&str> {
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
pub(crate) fn prompt_definition(snapshot: &ExecutionSnapshot, question: &PromptsCreateRequest) -> Result<Prompt> {
    if let Some(definition) = &question.definition {
        return Ok(definition.clone());
    }
    let catalog = h::Declarations::decode(snapshot.execution.catalog.as_slice())?;
    for section in catalog.sections {
        if section.r#type != "dokime.model.v1.PromptDefinitions" {
            continue;
        }
        for prompt in dokime_protocol::model::PromptDefinitions::decode(section.payload.as_slice())?.prompts {
            if prompt.metadata.as_ref().is_some_and(|metadata| metadata.identifier == question.prompt_type) {
                return Ok(prompt);
            }
        }
    }
    Err(Error::Invalid("Read Prompt Definition: Declare the Prompt in the Frozen Catalog".into()))
}
pub(crate) fn summary(snapshot: &ExecutionSnapshot, controls: ExecutionControls) -> ExecutionSummary {
    let state = match snapshot.state {
        ExecutionState::Open => dokime_protocol::host::ExecutionState::Open,
        ExecutionState::Closed => dokime_protocol::host::ExecutionState::Closed,
        ExecutionState::Sealed => dokime_protocol::host::ExecutionState::Sealed,
    };
    ExecutionSummary {
        execution_id: snapshot.execution.identifier.clone(),
        state: state as i32,
        outcome: snapshot.execution.outcome.clone(),
        controls: Some(controls),
        ..Default::default()
    }
}
pub(crate) fn counts<'a>(outcomes: impl IntoIterator<Item = &'a ExecutionOutcome>) -> ExecutionOutcomeCounts {
    let counts = OutcomeCounts::from_outcomes(outcomes);
    ExecutionOutcomeCounts {
        evaluated_count: counts.evaluated,
        not_evaluated_count: counts.not_evaluated,
        declared_count: counts.declared,
        result_status: counts.verdict.map(|verdict| verdict as i32),
    }
}
pub(crate) fn schemas(snapshot: &ExecutionSnapshot) -> Result<FrozenSchemas> {
    let catalog = h::Declarations::decode(snapshot.execution.catalog.as_slice())?;
    let mut records = Vec::new();
    for section in &catalog.sections {
        if section.r#type != "dokime.model.v1.RecordContracts" {
            continue;
        }
        for contract in dokime_protocol::model::RecordContracts::decode(section.payload.as_slice())?.records {
            records.push(dokime_protocol::host::RecordSchema {
                record_type: contract.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default(),
                schema_version: 1,
                metadata: contract.metadata,
                message_name: contract.message,
                ..Default::default()
            });
        }
    }
    let streams = catalog
        .streams
        .iter()
        .map(|stream| dokime_protocol::host::TelemetrySchema {
            stream: stream.id.clone(),
            message_name: stream.row.clone(),
            schema_version: 1,
            ..Default::default()
        })
        .collect();
    let digest = Sha256::digest(catalog.types.unwrap_or_default().encode_to_vec()).to_vec();
    Ok(FrozenSchemas { digest, records, streams })
}
pub(crate) fn records(snapshot: &ExecutionSnapshot, kinds: &[String], step: Option<&str>) -> Result<Vec<CapturedRecord>> {
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
        let record = RecordsAppendRequest::decode(evidence.payload.as_slice())?;
        let mut scope = scope(snapshot);
        scope.step_id = Some(evidence.step.clone());
        records.push(CapturedRecord {
            record_id: format!("{}:{}", snapshot.execution.identifier, evidence.position),
            sequence: evidence.position,
            timestamp: record.source_at,
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
/// The Snapshot's Samples, Decoded by Its Execution's Frozen Catalog.
pub(crate) fn samples(
    snapshot: &ExecutionSnapshot, catalog: &FrozenCatalog, streams: &[String], step: Option<&str>,
) -> Result<Vec<TelemetrySample>> {
    let pool = catalog
        .pool
        .as_ref()
        .ok_or_else(|| Error::Invalid("Read Samples Requires Frozen Descriptors".into()))?
        .as_ref()
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
            .declarations
            .streams
            .iter()
            .find(|stream| stream.id == sample.stream)
            .ok_or_else(|| Error::Invalid("Read Sample Requires a Frozen Stream Contract".into()))?;
        let descriptor =
            pool.get_message_by_name(&stream.row).ok_or_else(|| Error::Invalid("Read Sample Requires a Frozen Row Descriptor".into()))?;
        let row = DynamicMessage::decode(descriptor, sample.row.as_slice())?;
        let mut values = HashMap::new();
        for field in row.descriptor().fields() {
            values.insert(field.name().into(), value(row.get_field(&field).as_ref()));
        }
        samples.push(TelemetrySample {
            sample_id: format!("{}:{}", snapshot.execution.identifier, sample.position),
            stream: sample.stream.clone(),
            sequence: sample.sequence,
            schema_version: 1,
            timestamp: Some(timestamp(sample.source_ns)),
            received_at: Some(timestamp(sample.receipt_ns)),
            scope: Some(scope(snapshot)),
            values: Some(dokime_protocol::model::MapValue { values }),
            payload: sample.row.clone(),
            ..Default::default()
        });
    }
    Ok(samples)
}
pub(crate) fn prompt_id(execution: &str, prompt: &str, position: u64) -> String {
    format!("{execution}/{prompt}/{position}")
}
pub(crate) fn pending(context: &Context, identifier: &str) -> Result<(String, PromptsCreateRequest)> {
    for (execution, question) in context.host.pending()? {
        let snapshot = crate::host::execution_state::evidence(context, &execution)?;
        if let Some(entry) = snapshot.evidence.iter().rev().find(|entry| {
            entry.kind == "prompt"
                && PromptsCreateRequest::decode(entry.payload.as_slice()).is_ok_and(|stored| stored.prompt == question.prompt)
        }) && prompt_id(&execution, &question.prompt, entry.position) == identifier
        {
            return Ok((execution, question));
        }
    }
    Err(Error::Precondition("Answer Prompt: Select the Current Pending Question".into()))
}
/* === Private Functions === */

pub(crate) fn value(value: &Value) -> ProductValue {
    use Value as V;
    use dokime_protocol::model::value::Kind;
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
        V::List(values) => Kind::List(dokime_protocol::model::ValueList {
            values: values.iter().map(self::value).collect(),
        }),
        V::Map(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(key, _)| format!("{key:?}"));
            let values = entries
                .into_iter()
                .map(|(key, item)| {
                    let key = match key {
                        MapKey::Bool(key) => Kind::Boolean(*key),
                        MapKey::I32(key) => Kind::Integer(i64::from(*key)),
                        MapKey::I64(key) => Kind::Integer(*key),
                        MapKey::U32(key) => Kind::UnsignedInteger(u64::from(*key)),
                        MapKey::U64(key) => Kind::UnsignedInteger(*key),
                        MapKey::String(key) => Kind::Text(key.clone()),
                    };
                    dokime_protocol::model::Value {
                        kind: Some(Kind::List(dokime_protocol::model::ValueList {
                            values: vec![dokime_protocol::model::Value { kind: Some(key) }, self::value(item)],
                        })),
                    }
                })
                .collect();
            Kind::List(dokime_protocol::model::ValueList { values })
        }
    };
    dokime_protocol::model::Value { kind: Some(kind) }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::samples;
    use dokime_storage::{Execution, ExecutionSnapshot, ExecutionState, FrozenCatalog};
    use harmos_protocol::model::Declarations;
    use prost::Message;
    use prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet};

    #[test]
    fn empty_selections_answer_empty_but_still_require_frozen_descriptors() {
        let snapshot = ExecutionSnapshot {
            execution: Execution::default(),
            state: ExecutionState::Open,
            evidence: Vec::new(),
            samples: Vec::new(),
            seal: Vec::new(),
        };
        let untyped = FrozenCatalog::decode(&Declarations::default().encode_to_vec()).unwrap();
        assert!(samples(&snapshot, &untyped, &[], None).is_err());
        let typed = Declarations {
            types: Some(FileDescriptorSet {
                file: vec![FileDescriptorProto {
                    name: Some("fixture.proto".into()),
                    package: Some("fixture".into()),
                    message_type: vec![DescriptorProto {
                        name: Some("Row".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }),
            ..Default::default()
        };
        let typed = FrozenCatalog::decode(&typed.encode_to_vec()).unwrap();
        assert!(samples(&snapshot, &typed, &[], None).unwrap().is_empty());
    }
}

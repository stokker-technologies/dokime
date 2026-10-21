/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Project Bounded Native Values Into Canonical Messages

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use tonic::Status;

// Local Imports
use super::{Budget, collection, text};
use dokime::domain::model::definition::wire as schema;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::domain::model::identity::Metadata;
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::values::ElapsedUnit;
use dokime::domain::model::values::Value;
use dokime::domain::model::values::ValueFormat;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn decode(
    value: dokime::domain::model::values::wire::Value, budget: &mut Budget, depth: usize,
) -> Result<Value, Status> {
    preflight(&value, budget, depth)?;
    value.to_core().map_err(|error| Status::invalid_argument(error.to_string()))
}

pub(in crate::grpc) fn decode_metadata(value: &common::Metadata) -> Result<Metadata, Status> {
    text(&value.identifier)?;
    text(&value.label)?;
    if let Some(description) = &value.description {
        text(description)?;
    }
    collection(value.tags.len())?;
    collection(value.attributes.len())?;
    for tag in &value.tags {
        text(tag)?;
    }
    let mut budget = Budget::new();
    for attribute in &value.attributes {
        text(&attribute.key)?;
        let item = attribute.value.as_ref().ok_or_else(|| Status::invalid_argument("metadata attribute value is required"))?;
        preflight(item, &mut budget, 0)?;
    }
    Metadata::try_from(value).map_err(|error| Status::invalid_argument(error.to_string()))
}

pub(in crate::grpc) fn encode(
    value: &Value, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    native(value, budget, depth)?;
    Ok(dokime::domain::model::values::wire::Value::from(value))
}

pub(in crate::grpc) fn object(
    fields: &BTreeMap<String, Value>, budget: &mut Budget,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    object_at(fields, budget, 0)
}

pub(in crate::grpc) fn object_at(
    fields: &BTreeMap<String, Value>, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    collection(fields.len())?;
    for (key, value) in fields {
        text(key)?;
        native(value, budget, depth)?;
    }
    Ok(dokime::domain::model::values::wire::Value::from(fields))
}

pub(in crate::grpc) fn metadata(value: &Metadata) -> Result<common::Metadata, Status> {
    text(value.identifier.as_str())?;
    text(&value.label)?;
    if let Some(description) = &value.description {
        text(description)?;
    }
    collection(value.tags.len())?;
    collection(value.attributes.len())?;
    for tag in &value.tags {
        text(tag)?;
    }
    let mut budget = Budget::new();
    for (key, item) in &value.attributes {
        text(key)?;
        native(item, &mut budget, 0)?;
    }
    Ok(common::Metadata::from(value))
}

pub(in crate::grpc) fn cursor(snapshot: &dokime::runtime::query::RuntimeStateSnapshot) -> Cursor {
    Cursor { epoch: snapshot.state().process_epoch().to_string(), stream: "runtime.state".into(), position: snapshot.revision() }
}

pub(in crate::grpc) fn timestamp(value: &dokime::domain::model::time::Timestamp) -> Result<common::Timestamp, Status> {
    let at = value.to_datetime().map_err(|_| Status::internal("invalid native timestamp"))?;
    Ok(common::Timestamp { seconds: at.unix_timestamp(), nanos: at.nanosecond() as i32 })
}

pub(in crate::grpc) fn decoded_timestamp(value: common::Timestamp) -> Result<dokime::domain::model::time::Timestamp, Status> {
    let at = value.to_datetime().map_err(|_| Status::invalid_argument("invalid timestamp"))?;
    Ok(dokime::domain::model::time::Timestamp::from_datetime(at))
}

pub(in crate::grpc) fn scope(value: &dokime::domain::model::execution::RecordScope) -> Result<common::EvidenceScope, Status> {
    scope_memory(value)?;
    let ids = scope_ids(value);
    let origin = match &value.origin {
        dokime::domain::model::execution::EvidenceOrigin::Default => common::EvidenceOriginKind::Default(true),
        dokime::domain::model::execution::EvidenceOrigin::Initialization => common::EvidenceOriginKind::Initialization(true),
        dokime::domain::model::execution::EvidenceOrigin::Step { attempt_id } => {
            text(attempt_id.as_str())?;
            common::EvidenceOriginKind::StepAttemptId(attempt_id.to_string())
        }
        dokime::domain::model::execution::EvidenceOrigin::Cleanup => common::EvidenceOriginKind::Cleanup(true),
    };
    Ok(common::EvidenceScope {
        session_id: ids[0].map(str::to_owned),
        cycle_id: ids[1].map(str::to_owned),
        case_id: ids[2].map(str::to_owned),
        execution_id: ids[3].map(str::to_owned),
        step_id: ids[4].map(str::to_owned),
        criterion_id: ids[5].map(str::to_owned),
        template_id: ids[6].map(str::to_owned),
        suite_id: ids[7].map(str::to_owned),
        origin: Some(common::EvidenceOrigin { kind: Some(origin) }),
    })
}

pub(in crate::grpc) fn scope_memory(value: &dokime::domain::model::execution::RecordScope) -> Result<usize, Status> {
    let mut bytes = std::mem::size_of::<common::EvidenceScope>();
    for id in scope_ids(value).into_iter().flatten() {
        text(id)?;
        bytes = bytes.saturating_add(id.len());
    }
    if let dokime::domain::model::execution::EvidenceOrigin::Step { attempt_id } = &value.origin {
        text(attempt_id.as_str())?;
        bytes = bytes.saturating_add(attempt_id.as_str().len());
    }
    bounded(bytes)
}

pub(in crate::grpc) fn decoded_scope(scope: &common::EvidenceScope) -> Result<dokime::domain::model::execution::RecordScope, Status> {
    use dokime::domain::model::execution::EvidenceOrigin;
    use dokime::domain::model::execution::RecordScope;
    use dokime::domain::model::identity::CaseId;
    use dokime::domain::model::identity::CriterionId;
    use dokime::domain::model::identity::SessionId;
    use dokime::domain::model::identity::StepAttemptId;
    use dokime::domain::model::identity::StepId;
    use dokime::domain::model::identity::SuiteId;
    use dokime::domain::model::identity::TemplateId;
    use dokime::domain::model::identity::TestCycleId;
    use dokime::domain::model::identity::TestExecutionId;
    let origin = match scope.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        None | Some(common::EvidenceOriginKind::Default(true)) => EvidenceOrigin::Default,
        Some(common::EvidenceOriginKind::Initialization(true)) => EvidenceOrigin::Initialization,
        Some(common::EvidenceOriginKind::Cleanup(true)) => EvidenceOrigin::Cleanup,
        Some(common::EvidenceOriginKind::StepAttemptId(id)) => {
            text(id)?;
            EvidenceOrigin::Step { attempt_id: StepAttemptId::from_uuid(id).map_err(|error| Status::invalid_argument(error.to_string()))? }
        }
        _ => return Err(Status::invalid_argument("invalid evidence origin")),
    };
    if origin != EvidenceOrigin::Default && scope.execution_id.is_none() {
        return Err(Status::invalid_argument("evidence origin requires an execution owner"));
    }
    let test_case_id = scope
        .case_id
        .as_deref()
        .map(|id| {
            text(id)?;
            CaseId::from_wire(id).map_err(|error| Status::invalid_argument(format!("invalid case_id: {error}")))
        })
        .transpose()?;
    Ok(RecordScope {
        origin,
        session_id: scope_id(scope.session_id.as_deref(), "session_id", |id| SessionId::from_uuid(id))?,
        test_cycle_id: scope_id(scope.cycle_id.as_deref(), "cycle_id", |id| TestCycleId::from_uuid(id))?,
        test_case_id,
        test_execution_id: scope_id(scope.execution_id.as_deref(), "execution_id", |id| TestExecutionId::from_uuid(id))?,
        step_id: scope_id(scope.step_id.as_deref(), "step_id", |id| StepId::from_uuid(id))?,
        criterion_id: scope_id(scope.criterion_id.as_deref(), "criterion_id", |id| CriterionId::from_uuid(id))?,
        template_id: scope_id(scope.template_id.as_deref(), "template_id", |id| TemplateId::from_uuid(id))?,
        suite_id: scope_id(scope.suite_id.as_deref(), "suite_id", |id| SuiteId::from_uuid(id))?,
    })
}

pub(in crate::grpc) fn execution_plugin_memory(
    value: &dokime::domain::model::execution::snapshot::ExecutionPluginIdentity,
) -> Result<usize, Status> {
    use dokime::domain::model::execution::snapshot::ExecutionImplementation;
    text(&value.source)?;
    if let Some(version) = &value.version {
        text(version)?;
    }
    let implementation = match &value.implementation {
        ExecutionImplementation::Bundle { content_digest } => content_digest,
        ExecutionImplementation::Embedded { generation } => generation,
    };
    text(implementation)?;
    bounded(
        std::mem::size_of::<dokime::domain::model::execution::wire::ExecutionPluginIdentity>()
            .saturating_add(value.source.len())
            .saturating_add(value.version.as_ref().map_or(0, String::len))
            .saturating_add(implementation.len()),
    )
}

pub(in crate::grpc) fn execution_plugin(
    value: &dokime::domain::model::execution::snapshot::ExecutionPluginIdentity,
) -> Result<dokime::domain::model::execution::wire::ExecutionPluginIdentity, Status> {
    use dokime::domain::model::execution::snapshot::ExecutionImplementation;
    use dokime::domain::model::execution::wire::ExecutionPluginIdentity;
    use dokime::domain::model::execution::wire::ExecutionPluginIdentityImplementation as Implementation;
    text(&value.source)?;
    if let Some(version) = &value.version {
        text(version)?;
    }
    let implementation = match &value.implementation {
        ExecutionImplementation::Bundle { content_digest } => {
            text(content_digest)?;
            Implementation::ContentDigest(content_digest.clone())
        }
        ExecutionImplementation::Embedded { generation } => {
            text(generation)?;
            Implementation::Generation(generation.clone())
        }
    };
    Ok(ExecutionPluginIdentity { source: value.source.clone(), version: value.version.clone(), implementation: Some(implementation) })
}

pub(in crate::grpc) fn memory(value: &Value, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    native(value, budget, depth)?;
    let (nodes, heap) = heap(value);
    // Conservative Vec Growth: Include Four Slots Per Node, Including The Initial Allocation.
    bounded(
        std::mem::size_of::<dokime::domain::model::values::wire::Value>()
            .saturating_add(nodes.saturating_mul(4).saturating_mul(std::mem::size_of::<dokime::domain::model::values::wire::ValueNode>()))
            .saturating_add(heap),
    )
}

pub(in crate::grpc) fn object_memory(fields: &BTreeMap<String, Value>, budget: &mut Budget) -> Result<usize, Status> {
    object_memory_at(fields, budget, 0)
}

pub(in crate::grpc) fn object_memory_at(fields: &BTreeMap<String, Value>, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    collection(fields.len())?;
    let mut bytes = std::mem::size_of::<dokime::domain::model::values::wire::Value>()
        .saturating_add(4 * std::mem::size_of::<dokime::domain::model::values::wire::ValueNode>())
        .saturating_add(
            fields.len().saturating_mul(4).saturating_mul(std::mem::size_of::<dokime::domain::model::values::wire::ValueField>()),
        );
    for (key, value) in fields {
        text(key)?;
        bytes = bytes
            .saturating_add(key.len())
            .saturating_add(memory(value, budget, depth)? - std::mem::size_of::<dokime::domain::model::values::wire::Value>());
    }
    bounded(bytes)
}

pub(in crate::grpc) fn metadata_memory(value: &Metadata) -> Result<usize, Status> {
    text(value.identifier.as_str())?;
    text(&value.label)?;
    if let Some(description) = &value.description {
        text(description)?;
    }
    collection(value.tags.len())?;
    collection(value.attributes.len())?;
    let mut bytes = std::mem::size_of::<common::Metadata>()
        .saturating_add(value.identifier.as_str().len())
        .saturating_add(value.label.len())
        .saturating_add(value.description.as_ref().map_or(0, String::len))
        .saturating_add(value.tags.len().saturating_mul(std::mem::size_of::<String>()))
        .saturating_add(value.attributes.len().saturating_mul(4).saturating_mul(std::mem::size_of::<common::MetadataAttribute>()));
    for tag in &value.tags {
        text(tag)?;
        bytes = bytes.saturating_add(tag.len());
    }
    let mut budget = Budget::new();
    for (key, value) in &value.attributes {
        text(key)?;
        bytes = bytes
            .saturating_add(key.len())
            .saturating_add(memory(value, &mut budget, 0)? - std::mem::size_of::<dokime::domain::model::values::wire::Value>());
    }
    bounded(bytes)
}

pub(in crate::grpc) fn slots<T>(length: usize) -> Result<Vec<T>, Status> {
    collection(length)?;
    let mut values = Vec::new();
    values.try_reserve_exact(length).map_err(|_| Status::resource_exhausted("canonical projection allocation refused"))?;
    Ok(values)
}

pub(in crate::grpc) fn value_type(value: &dokime::domain::model::values::ValueType) -> schema::ValueType {
    use dokime::domain::model::values::ValueType;
    match value {
        ValueType::Boolean => schema::ValueType::Boolean,
        ValueType::Integer => schema::ValueType::Integer,
        ValueType::Number => schema::ValueType::Number,
        ValueType::Text => schema::ValueType::Text,
        ValueType::File => schema::ValueType::File,
        ValueType::Object => schema::ValueType::Object,
        ValueType::Array => schema::ValueType::Array,
        ValueType::Datetime => schema::ValueType::Datetime,
    }
}

pub(in crate::grpc) fn value_format(value: &dokime::domain::model::values::ValueFormat) -> schema::ValueFormat {
    use schema::ValueFormatKind as Kind;
    schema::ValueFormat {
        kind: Some(match value {
            ValueFormat::Normal { max_fraction_digits } => {
                Kind::Normal(schema::NormalFormat { max_fraction_digits: max_fraction_digits.map(u32::from) })
            }
            ValueFormat::Fixed { decimals } => Kind::Fixed(schema::FixedFormat { decimals: u32::from(*decimals) }),
            ValueFormat::Scientific { significant_digits } => {
                Kind::Scientific(schema::ScientificFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::Engineering { significant_digits } => {
                Kind::Engineering(schema::EngineeringFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::SiPrefix { significant_digits } => {
                Kind::SiPrefix(schema::SiPrefixFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::AbsoluteTime { pattern } => Kind::AbsoluteTime(schema::AbsoluteTimeFormat { pattern: pattern.clone() }),
            ValueFormat::ElapsedSinceRun { unit } => Kind::ElapsedSinceRun(schema::ElapsedFormat { unit: elapsed_unit(unit) }),
            ValueFormat::ElapsedSinceCycle { unit } => Kind::ElapsedSinceCycle(schema::ElapsedFormat { unit: elapsed_unit(unit) }),
        }),
    }
}

pub(in crate::grpc) fn cursor_failure(error: dokime::domain::model::execution::wire_cursor::Error) -> Status {
    use dokime::domain::model::execution::wire_cursor::Error;
    let code = match error {
        Error::Selection => tonic::Code::FailedPrecondition,
        Error::Ahead => tonic::Code::OutOfRange,
        Error::Changed => tonic::Code::Aborted,
        Error::AfterTooLarge => tonic::Code::ResourceExhausted,
        Error::Limit | Error::Missing => tonic::Code::InvalidArgument,
    };
    Status::new(code, error.to_string())
}

pub(in crate::grpc) fn format(value: &ValueFormat) -> schema::ValueFormat {
    use schema::ValueFormatKind as Kind;
    schema::ValueFormat {
        kind: Some(match value {
            ValueFormat::Normal { max_fraction_digits } => {
                Kind::Normal(schema::NormalFormat { max_fraction_digits: max_fraction_digits.map(u32::from) })
            }
            ValueFormat::Fixed { decimals } => Kind::Fixed(schema::FixedFormat { decimals: u32::from(*decimals) }),
            ValueFormat::Scientific { significant_digits } => {
                Kind::Scientific(schema::ScientificFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::Engineering { significant_digits } => {
                Kind::Engineering(schema::EngineeringFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::SiPrefix { significant_digits } => {
                Kind::SiPrefix(schema::SiPrefixFormat { significant_digits: u32::from(*significant_digits) })
            }
            ValueFormat::AbsoluteTime { pattern } => Kind::AbsoluteTime(schema::AbsoluteTimeFormat { pattern: pattern.clone() }),
            ValueFormat::ElapsedSinceRun { unit } => Kind::ElapsedSinceRun(schema::ElapsedFormat { unit: elapsed_unit(unit) }),
            ValueFormat::ElapsedSinceCycle { unit } => Kind::ElapsedSinceCycle(schema::ElapsedFormat { unit: elapsed_unit(unit) }),
        }),
    }
}

/* === Private Functions === */

fn elapsed_unit(value: &ElapsedUnit) -> schema::ElapsedUnit {
    match value {
        ElapsedUnit::Seconds => schema::ElapsedUnit::Seconds,
        ElapsedUnit::Milliseconds => schema::ElapsedUnit::Milliseconds,
        ElapsedUnit::Minutes => schema::ElapsedUnit::Minutes,
    }
}

fn scope_id<T>(value: Option<&str>, label: &str, build: impl FnOnce(&str) -> Result<T, dokime::ApiError>) -> Result<Option<T>, Status> {
    let Some(value) = value else {
        return Ok(None);
    };
    text(value)?;
    if uuid::Uuid::parse_str(value).map(|value| value.is_nil()).unwrap_or(true) {
        return Err(Status::invalid_argument(format!("invalid {label}")));
    }
    build(value).map(Some).map_err(|error| Status::invalid_argument(format!("invalid {label}: {error}")))
}

fn scope_ids(value: &dokime::domain::model::execution::RecordScope) -> [Option<&str>; 8] {
    [
        value.session_id.as_ref().map(|id| id.as_str()),
        value.test_cycle_id.as_ref().map(|id| id.as_str()),
        value.test_case_id.as_ref().map(|id| id.as_str()),
        value.test_execution_id.as_ref().map(|id| id.as_str()),
        value.step_id.as_ref().map(|id| id.as_str()),
        value.criterion_id.as_ref().map(|id| id.as_str()),
        value.template_id.as_ref().map(|id| id.as_str()),
        value.suite_id.as_ref().map(|id| id.as_str()),
    ]
}

fn preflight(value: &dokime::domain::model::values::wire::Value, budget: &mut Budget, depth: usize) -> Result<(), Status> {
    if value.nodes.len() > budget.0 {
        return Err(Status::resource_exhausted("value/expression budget exceeded"));
    }
    // Preflight Allocation/Depth/Text Before The Canonical Graph Rebuilder Allocates Values.
    let mut depths = vec![depth; value.nodes.len()];
    for (index, node) in value.nodes.iter().enumerate() {
        if depths[index] > 32 {
            return Err(Status::resource_exhausted("value/expression budget exceeded"));
        }
        let mut child = |member: u32| -> Result<(), Status> {
            let member = member as usize;
            if member <= index || member >= value.nodes.len() {
                return Err(Status::invalid_argument("invalid value member index"));
            }
            depths[member] = depths[index] + 1;
            Ok(())
        };
        match node.kind.as_ref() {
            Some(dokime::domain::model::values::wire::ValueKind::Text(value)) => text(value)?,
            Some(dokime::domain::model::values::wire::ValueKind::Array(values)) => {
                collection(values.items.len())?;
                for member in &values.items {
                    child(*member)?;
                }
            }
            Some(dokime::domain::model::values::wire::ValueKind::Object(values)) => {
                collection(values.fields.len())?;
                for field in &values.fields {
                    text(&field.key)?;
                    child(field.value)?;
                }
            }
            _ => {}
        }
    }
    budget.0 -= value.nodes.len();
    Ok(())
}

fn bounded(bytes: usize) -> Result<usize, Status> {
    if bytes == usize::MAX { Err(Status::resource_exhausted("canonical projection memory overflows")) } else { Ok(bytes) }
}

fn heap(value: &Value) -> (usize, usize) {
    match value {
        Value::Text(value) => (1, value.len()),
        Value::Array(values) => {
            let mut nodes = 1usize;
            let mut bytes = values.len().saturating_mul(std::mem::size_of::<u32>());
            for value in values {
                let (count, heap) = heap(value);
                nodes = nodes.saturating_add(count);
                bytes = bytes.saturating_add(heap);
            }
            (nodes, bytes)
        }
        Value::Object(values) => {
            let mut nodes = 1usize;
            let mut bytes =
                values.len().saturating_mul(4).saturating_mul(std::mem::size_of::<dokime::domain::model::values::wire::ValueField>());
            for (key, value) in values {
                let (count, heap) = heap(value);
                nodes = nodes.saturating_add(count);
                bytes = bytes.saturating_add(key.len()).saturating_add(heap);
            }
            (nodes, bytes)
        }
        _ => (1, 0),
    }
}

fn native(value: &Value, budget: &mut Budget, depth: usize) -> Result<(), Status> {
    budget.node(depth)?;
    match value {
        Value::Text(value) => text(value)?,
        Value::Array(values) => {
            collection(values.len())?;
            for value in values {
                native(value, budget, depth + 1)?;
            }
        }
        Value::Object(values) => {
            collection(values.len())?;
            for (key, value) in values {
                text(key)?;
                native(value, budget, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/* === Tests === */

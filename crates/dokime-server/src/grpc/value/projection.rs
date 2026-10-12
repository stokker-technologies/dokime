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
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

// Third Party Imports
use prost::Message;

// Local Imports
use super::Budget;
use dokime_app::core::{EvidenceOrigin, Metadata, RecordScope, Value};
use dokime_protocol::v1;

/* === Constants === */
include!(concat!(env!("OUT_DIR"), "/value-output-fields.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(in crate::grpc) struct Count {
    pub(in crate::grpc) memory: usize,
    pub(in crate::grpc) encoded: usize,
}

struct TextCount(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Write for TextCount {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("datetime length overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */
pub(in crate::grpc) fn count(
    value: &Value,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<Count> {
    budget.node(depth).map_err(|_| exhausted())?;
    let inline = std::mem::size_of::<v1::Value>();
    let (heap, encoded) = match value {
        Value::Text(value) => {
            super::text(value).map_err(|_| exhausted())?;
            (
                value.len(),
                prost::encoding::string::encoded_len(TEXT, value),
            )
        }
        Value::Datetime(value) => {
            let length = datetime_length(*value)?;
            (length, framed(DATETIME, length)?)
        }
        Value::Array(values) => {
            super::collection(values.len()).map_err(|_| exhausted())?;
            let mut heap = values.len().checked_mul(inline).ok_or_else(exhausted)?;
            let mut encoded = 0usize;
            for value in values {
                let count = count(value, budget, depth + 1)?;
                heap = heap
                    .checked_add(count.memory - inline)
                    .ok_or_else(exhausted)?;
                encoded = encoded
                    .checked_add(framed(LIST_VALUES, count.encoded)?)
                    .ok_or_else(exhausted)?;
            }
            (heap, framed(ARRAY, encoded)?)
        }
        Value::Object(values) => {
            let object = object_count(values, budget, depth)?;
            (
                object.memory - std::mem::size_of::<v1::ValueObject>(),
                framed(OBJECT, object.encoded)?,
            )
        }
        _ => (0, scalar(value)?.encoded_len()),
    };
    Ok(Count {
        memory: inline.checked_add(heap).ok_or_else(exhausted)?,
        encoded,
    })
}

pub(in crate::grpc) fn project(
    value: &Value,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<v1::Value> {
    budget.node(depth).map_err(|_| exhausted())?;
    let kind = match value {
        Value::Text(value) => v1::value::Kind::TextValue(text(value)?),
        Value::Datetime(value) => v1::value::Kind::DatetimeValue(datetime_text(*value)?),
        Value::Array(values) => {
            let mut target = slots(values.len())?;
            for value in values {
                target.push(project(value, budget, depth + 1)?);
            }
            v1::value::Kind::ArrayValue(v1::ValueList { values: target })
        }
        Value::Object(values) => {
            let mut fields = BTreeMap::new();
            for (key, value) in values {
                fields.insert(text(key)?, project(value, budget, depth + 1)?);
            }
            v1::value::Kind::ObjectValue(v1::ValueObject { fields })
        }
        _ => return scalar(value),
    };
    Ok(v1::Value { kind: Some(kind) })
}

pub(in crate::grpc) fn object_count(
    values: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<Count> {
    super::collection(values.len()).map_err(|_| exhausted())?;
    let nodes = harmos::btree_map_memory::<String, v1::Value>(values.len());
    if nodes == usize::MAX {
        return Err(exhausted());
    }
    let mut memory = std::mem::size_of::<v1::ValueObject>()
        .checked_add(nodes - std::mem::size_of::<BTreeMap<String, v1::Value>>())
        .ok_or_else(exhausted)?;
    let mut encoded = 0usize;
    for (key, value) in values {
        super::text(key).map_err(|_| exhausted())?;
        let value = count(value, budget, depth + 1)?;
        memory = memory
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(value.memory - std::mem::size_of::<v1::Value>()))
            .ok_or_else(exhausted)?;
        let key = if key.is_empty() {
            0
        } else {
            framed(1, key.len())?
        };
        let entry = key
            .checked_add(framed(2, value.encoded)?)
            .ok_or_else(exhausted)?;
        encoded = encoded
            .checked_add(framed(OBJECT_FIELDS, entry)?)
            .ok_or_else(exhausted)?;
    }
    Ok(Count { memory, encoded })
}

pub(in crate::grpc) fn object(
    values: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<v1::ValueObject> {
    let mut fields = BTreeMap::new();
    for (key, value) in values {
        fields.insert(text(key)?, project(value, budget, depth + 1)?);
    }
    Ok(v1::ValueObject { fields })
}

pub(in crate::grpc) fn scope_memory(source: &RecordScope) -> usize {
    let mut memory = std::mem::size_of::<v1::EvidenceScope>();
    for field in [
        source.session_id.as_ref().map(|id| id.as_str()),
        source.test_cycle_id.as_ref().map(|id| id.as_str()),
        source.test_case_id.as_ref().map(|id| id.as_str()),
        source.test_execution_id.as_ref().map(|id| id.as_str()),
        source.step_id.as_ref().map(|id| id.as_str()),
        source.criterion_id.as_ref().map(|id| id.as_str()),
        source.template_id.as_ref().map(|id| id.as_str()),
        source.suite_id.as_ref().map(|id| id.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        memory = memory.saturating_add(field.len());
    }
    if let EvidenceOrigin::Step { attempt_id } = &source.origin {
        memory = memory.saturating_add(attempt_id.as_str().len());
    }
    memory
}

pub(in crate::grpc) fn scope(source: &RecordScope) -> harmos::Result<v1::EvidenceScope> {
    let origin = match &source.origin {
        EvidenceOrigin::Default => v1::evidence_origin::Kind::Default(true),
        EvidenceOrigin::Initialization => v1::evidence_origin::Kind::Initialization(true),
        EvidenceOrigin::Step { attempt_id } => {
            v1::evidence_origin::Kind::StepAttemptId(text(attempt_id.as_str())?)
        }
        EvidenceOrigin::Cleanup => v1::evidence_origin::Kind::Cleanup(true),
    };
    Ok(v1::EvidenceScope {
        session_id: source
            .session_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        cycle_id: source
            .test_cycle_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        case_id: source
            .test_case_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        execution_id: source
            .test_execution_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        step_id: source
            .step_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        criterion_id: source
            .criterion_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        template_id: source
            .template_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        suite_id: source
            .suite_id
            .as_ref()
            .map(|id| text(id.as_str()))
            .transpose()?,
        origin: Some(v1::EvidenceOrigin { kind: Some(origin) }),
    })
}

pub(in crate::grpc) fn metadata_count(source: &Metadata) -> harmos::Result<Count> {
    for field in [source.identifier.as_str(), &source.label] {
        super::text(field).map_err(|_| exhausted())?;
    }
    if let Some(description) = &source.description {
        super::text(description).map_err(|_| exhausted())?;
    }
    metadata_fields_count(source)
}

pub(in crate::grpc) fn metadata_fields_count(source: &Metadata) -> harmos::Result<Count> {
    super::collection(source.tags.len()).map_err(|_| exhausted())?;
    super::collection(source.attributes.len()).map_err(|_| exhausted())?;
    let mut memory = std::mem::size_of::<v1::Metadata>()
        .checked_add(source.identifier.as_str().len())
        .and_then(|bytes| bytes.checked_add(source.label.len()))
        .and_then(|bytes| bytes.checked_add(source.description.as_ref().map_or(0, String::len)))
        .and_then(|bytes| {
            bytes.checked_add(
                source
                    .tags
                    .len()
                    .checked_mul(std::mem::size_of::<String>())?,
            )
        })
        .ok_or_else(exhausted)?;
    let mut encoded = 0usize;
    for (tag, field) in [
        (METADATA_ID, source.identifier.as_str()),
        (METADATA_LABEL, source.label.as_str()),
    ] {
        if !field.is_empty() {
            encoded = encoded
                .checked_add(framed(tag, field.len())?)
                .ok_or_else(exhausted)?;
        }
    }
    if let Some(description) = &source.description {
        encoded = encoded
            .checked_add(framed(METADATA_DESCRIPTION, description.len())?)
            .ok_or_else(exhausted)?;
    }
    for tag in &source.tags {
        super::text(tag).map_err(|_| exhausted())?;
        memory = memory.checked_add(tag.len()).ok_or_else(exhausted)?;
        encoded = encoded
            .checked_add(framed(METADATA_TAGS, tag.len())?)
            .ok_or_else(exhausted)?;
    }
    let nodes = harmos::btree_map_memory::<String, v1::Value>(source.attributes.len());
    if nodes == usize::MAX {
        return Err(exhausted());
    }
    memory = memory
        .checked_add(nodes - std::mem::size_of::<BTreeMap<String, v1::Value>>())
        .ok_or_else(exhausted)?;
    let mut budget = Budget::new();
    for (key, value) in &source.attributes {
        super::text(key).map_err(|_| exhausted())?;
        let value = count(value, &mut budget, 0)?;
        memory = memory
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(value.memory - std::mem::size_of::<v1::Value>()))
            .ok_or_else(exhausted)?;
        let key = if key.is_empty() {
            0
        } else {
            framed(1, key.len())?
        };
        let entry = key
            .checked_add(framed(2, value.encoded)?)
            .ok_or_else(exhausted)?;
        encoded = encoded
            .checked_add(framed(METADATA_ATTRIBUTES, entry)?)
            .ok_or_else(exhausted)?;
    }
    for (tag, timestamp) in [
        (METADATA_CREATED, source.created_at),
        (METADATA_UPDATED, source.updated_at),
    ] {
        if let Some(timestamp) = timestamp {
            encoded = encoded
                .checked_add(framed(
                    tag,
                    super::datetime_timestamp(timestamp).encoded_len(),
                )?)
                .ok_or_else(exhausted)?;
        }
    }
    Ok(Count { memory, encoded })
}

pub(in crate::grpc) fn metadata(source: &Metadata) -> harmos::Result<v1::Metadata> {
    let mut tags = slots(source.tags.len())?;
    for tag in &source.tags {
        tags.push(text(tag)?);
    }
    let mut attributes = BTreeMap::new();
    let mut budget = Budget::new();
    for (key, value) in &source.attributes {
        attributes.insert(text(key)?, project(value, &mut budget, 0)?);
    }
    Ok(v1::Metadata {
        identifier: text(source.identifier.as_str())?,
        label: text(&source.label)?,
        description: source.description.as_deref().map(text).transpose()?,
        tags,
        attributes: attributes.into_iter().collect(),
        created_at: source.created_at.map(super::datetime_timestamp),
        updated_at: source.updated_at.map(super::datetime_timestamp),
    })
}

pub(in crate::grpc) fn text(source: &str) -> harmos::Result<String> {
    let mut target = String::new();
    target
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if target.capacity() > source.len() {
        return Err(exhausted());
    }
    target.push_str(source);
    Ok(target)
}

pub(in crate::grpc) fn slots<T>(length: usize) -> harmos::Result<Vec<T>> {
    let mut target = Vec::new();
    target.try_reserve_exact(length).map_err(|_| exhausted())?;
    if target.capacity() > length {
        return Err(exhausted());
    }
    Ok(target)
}

pub(in crate::grpc) fn framed(tag: u32, length: usize) -> harmos::Result<usize> {
    prost::encoding::key_len(tag)
        .checked_add(prost::encoding::encoded_len_varint(length as u64))
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)
}

/* === Private Functions === */
fn scalar(value: &Value) -> harmos::Result<v1::Value> {
    let kind = match value {
        Value::Null => v1::value::Kind::NullValue(0),
        Value::Bool(value) => v1::value::Kind::BoolValue(*value),
        Value::Integer(value) => v1::value::Kind::IntegerValue(*value),
        Value::Unsigned(value) => v1::value::Kind::UnsignedValue(*value),
        Value::Number(value) => v1::value::Kind::NumberValue(*value),
        _ => return Err(exhausted()),
    };
    Ok(v1::Value { kind: Some(kind) })
}

fn datetime_length(value: time::OffsetDateTime) -> harmos::Result<usize> {
    let mut count = TextCount(0);
    value
        .to_offset(time::UtcOffset::UTC)
        .format_into(&mut count, &time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid_datetime())?;
    // Count Actual Writes; The Formatter May Omit Fractional Bytes From Its Return.
    Ok(count.0)
}

fn datetime_text(value: time::OffsetDateTime) -> harmos::Result<String> {
    let length = datetime_length(value)?;
    let mut bytes = slots(length)?;
    value
        .to_offset(time::UtcOffset::UTC)
        .format_into(&mut bytes, &time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid_datetime())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid_datetime());
    }
    String::from_utf8(bytes).map_err(|_| invalid_datetime())
}

fn invalid_datetime() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "canonical datetime",
        reason: "RFC3339 projection failed",
    }
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical value projection",
    }
}

/* === Tests === */

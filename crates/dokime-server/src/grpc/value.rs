/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod projection;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use tonic::Status;
// Local Imports
use dokime_app::core::{ElapsedUnit, Metadata, RecordScope, Timestamp, Value, ValueFormat};
use dokime_protocol::v1;

/* === Constants === */
pub(super) const MAX_COLLECTION: usize = 256;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Budget(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Budget {
    pub(super) fn new() -> Self {
        Self(4096)
    }
    pub(super) fn node(&mut self, depth: usize) -> Result<(), Status> {
        if depth > 32 || self.0 == 0 {
            return Err(Status::resource_exhausted(
                "value/expression budget exceeded",
            ));
        }
        self.0 -= 1;
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn collection(len: usize) -> Result<(), Status> {
    if len > MAX_COLLECTION {
        return Err(Status::resource_exhausted("collection exceeds 256 items"));
    }
    Ok(())
}

pub(super) fn text(value: &str) -> Result<(), Status> {
    if value.len() > 4096 {
        return Err(Status::resource_exhausted("text exceeds 4096 bytes"));
    }
    Ok(())
}

pub(super) fn checked_metadata(value: &Metadata) -> Result<v1::Metadata, Status> {
    text(value.identifier.as_str())?;
    text(&value.label)?;
    if let Some(description) = &value.description {
        text(description)?;
    }
    metadata(value)
}

pub(super) fn metadata(value: &Metadata) -> Result<v1::Metadata, Status> {
    let mut budget = Budget::new();
    collection(value.tags.len())?;
    collection(value.attributes.len())?;
    for tag in &value.tags {
        text(tag)?;
    }

    Ok(v1::Metadata {
        identifier: value.identifier.to_string(),
        label: value.label.clone(),
        description: value.description.clone(),
        tags: value.tags.clone(),
        attributes: value
            .attributes
            .iter()
            .map(|(key, item)| {
                text(key)?;
                Ok((key.clone(), encode(item, &mut budget, 0)?))
            })
            .collect::<Result<_, Status>>()?,
        created_at: value.created_at.map(datetime_timestamp),
        updated_at: value.updated_at.map(datetime_timestamp),
    })
}

pub(super) fn decode_metadata(value: v1::Metadata) -> Result<Metadata, Status> {
    text(&value.identifier)?;
    text(&value.label)?;
    if let Some(description) = &value.description {
        text(description)?;
    }
    collection(value.tags.len())?;
    for tag in &value.tags {
        text(tag)?;
    }
    let mut metadata = Metadata::definition(
        dokime_app::core::Identifier::new(value.identifier)
            .map_err(|_| Status::invalid_argument("invalid metadata identifier"))?,
        value.label,
        value.description,
        value.tags,
    );
    metadata.attributes = decode_object(Some(v1::ValueObject {
        fields: value.attributes.into_iter().collect(),
    }))?;
    let datetime = |value: prost_types::Timestamp| -> Result<time::OffsetDateTime, Status> {
        if !(0..1_000_000_000).contains(&value.nanos) {
            return Err(Status::invalid_argument("invalid timestamp nanoseconds"));
        }
        time::OffsetDateTime::from_unix_timestamp(value.seconds)
            .and_then(|date| date.replace_nanosecond(value.nanos as u32))
            .map_err(|_| Status::invalid_argument("invalid timestamp"))
    };
    metadata.created_at = value.created_at.map(datetime).transpose()?;
    metadata.updated_at = value.updated_at.map(datetime).transpose()?;
    Ok(metadata)
}

pub(super) fn decode_object(
    value: Option<v1::ValueObject>,
) -> Result<BTreeMap<String, Value>, Status> {
    let fields = value.unwrap_or_default().fields;
    let mut budget = Budget::new();
    collection(fields.len())?;
    fields
        .into_iter()
        .map(|(key, value)| {
            text(&key)?;
            Ok((key, decode(value, &mut budget, 0)?))
        })
        .collect()
}

pub(super) fn decode(value: v1::Value, budget: &mut Budget, depth: usize) -> Result<Value, Status> {
    use v1::value::Kind;
    budget.node(depth)?;
    Ok(
        match value
            .kind
            .ok_or_else(|| Status::invalid_argument("Value.kind must be set"))?
        {
            Kind::NullValue(0) => Value::Null,
            Kind::NullValue(_) => return Err(Status::invalid_argument("unknown null enum value")),
            Kind::BoolValue(value) => Value::Bool(value),
            Kind::IntegerValue(value) => Value::Integer(value),
            Kind::UnsignedValue(value) => Value::Unsigned(value),
            Kind::NumberValue(value) => Value::Number(value),
            Kind::TextValue(value) => {
                text(&value)?;
                Value::Text(value)
            }
            Kind::DatetimeValue(value) => {
                text(&value)?;
                Value::Datetime(
                    time::OffsetDateTime::parse(
                        &value,
                        &time::format_description::well_known::Rfc3339,
                    )
                    .map_err(|_| Status::invalid_argument("invalid RFC3339 datetime"))?
                    .to_offset(time::UtcOffset::UTC),
                )
            }
            Kind::ArrayValue(value) => {
                collection(value.values.len())?;
                Value::Array(
                    value
                        .values
                        .into_iter()
                        .map(|value| decode(value, budget, depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            }
            Kind::ObjectValue(value) => {
                collection(value.fields.len())?;
                Value::Object(
                    value
                        .fields
                        .into_iter()
                        .map(|(key, value)| {
                            text(&key)?;
                            Ok((key, decode(value, budget, depth + 1)?))
                        })
                        .collect::<Result<_, Status>>()?,
                )
            }
        },
    )
}

pub(super) fn encode(
    value: &Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::Value, Status> {
    use v1::value::Kind;
    budget.node(depth)?;
    Ok(v1::Value {
        kind: Some(match value {
            Value::Null => Kind::NullValue(0),
            Value::Bool(value) => Kind::BoolValue(*value),
            Value::Integer(value) => Kind::IntegerValue(*value),
            Value::Unsigned(value) => Kind::UnsignedValue(*value),
            Value::Number(value) => Kind::NumberValue(*value),
            Value::Text(value) => {
                text(value)?;
                Kind::TextValue(value.clone())
            }
            Value::Datetime(value) => Kind::DatetimeValue(
                value
                    .to_offset(time::UtcOffset::UTC)
                    .format(&time::format_description::well_known::Rfc3339)
                    .map_err(|_| Status::internal("datetime outside RFC3339 range"))?,
            ),
            Value::Array(values) => {
                collection(values.len())?;
                Kind::ArrayValue(v1::ValueList {
                    values: values
                        .iter()
                        .map(|value| encode(value, budget, depth + 1))
                        .collect::<Result<_, _>>()?,
                })
            }
            Value::Object(values) => {
                collection(values.len())?;
                Kind::ObjectValue(v1::ValueObject {
                    fields: values
                        .iter()
                        .map(|(key, value)| {
                            text(key)?;
                            Ok((key.clone(), encode(value, budget, depth + 1)?))
                        })
                        .collect::<Result<_, Status>>()?,
                })
            }
        }),
    })
}

pub(super) fn scope(value: &RecordScope) -> v1::EvidenceScope {
    v1::EvidenceScope {
        session_id: value.session_id.as_ref().map(ToString::to_string),
        cycle_id: value.test_cycle_id.as_ref().map(ToString::to_string),
        case_id: value.test_case_id.as_ref().map(ToString::to_string),
        execution_id: value.test_execution_id.as_ref().map(ToString::to_string),
        step_id: value.step_id.as_ref().map(ToString::to_string),
        criterion_id: value.criterion_id.as_ref().map(ToString::to_string),
        template_id: value.template_id.as_ref().map(ToString::to_string),
        suite_id: value.suite_id.as_ref().map(ToString::to_string),
        origin: Some(v1::EvidenceOrigin {
            kind: Some(match &value.origin {
                dokime_app::core::EvidenceOrigin::Default => {
                    v1::evidence_origin::Kind::Default(true)
                }
                dokime_app::core::EvidenceOrigin::Initialization => {
                    v1::evidence_origin::Kind::Initialization(true)
                }
                dokime_app::core::EvidenceOrigin::Step { attempt_id } => {
                    v1::evidence_origin::Kind::StepAttemptId(attempt_id.to_string())
                }
                dokime_app::core::EvidenceOrigin::Cleanup => {
                    v1::evidence_origin::Kind::Cleanup(true)
                }
            }),
        }),
    }
}

pub(super) fn value_format(value: &ValueFormat) -> v1::ValueFormat {
    use v1::value_format::Kind;
    v1::ValueFormat {
        kind: Some(match value {
            ValueFormat::Normal {
                max_fraction_digits,
            } => Kind::Normal(v1::NormalFormat {
                max_fraction_digits: max_fraction_digits.map(u32::from),
            }),
            ValueFormat::Fixed { decimals } => Kind::Fixed(v1::FixedFormat {
                decimals: u32::from(*decimals),
            }),
            ValueFormat::Scientific { significant_digits } => {
                Kind::Scientific(v1::ScientificFormat {
                    significant_digits: u32::from(*significant_digits),
                })
            }
            ValueFormat::Engineering { significant_digits } => {
                Kind::Engineering(v1::EngineeringFormat {
                    significant_digits: u32::from(*significant_digits),
                })
            }
            ValueFormat::SiPrefix { significant_digits } => Kind::SiPrefix(v1::SiPrefixFormat {
                significant_digits: u32::from(*significant_digits),
            }),
            ValueFormat::AbsoluteTime { pattern } => Kind::AbsoluteTime(v1::AbsoluteTimeFormat {
                pattern: pattern.clone(),
            }),
            ValueFormat::ElapsedSinceRun { unit } => Kind::ElapsedSinceRun(v1::ElapsedFormat {
                unit: elapsed_unit(unit) as i32,
            }),
            ValueFormat::ElapsedSinceCycle { unit } => Kind::ElapsedSinceCycle(v1::ElapsedFormat {
                unit: elapsed_unit(unit) as i32,
            }),
        }),
    }
}

pub(super) fn optional_value_format(value: Option<&ValueFormat>) -> Option<v1::ValueFormat> {
    value.map(value_format)
}

pub(super) fn decoded_timestamp(value: prost_types::Timestamp) -> Result<Timestamp, Status> {
    if !(0..1_000_000_000).contains(&value.nanos) {
        return Err(Status::invalid_argument("invalid timestamp nanoseconds"));
    }
    time::OffsetDateTime::from_unix_timestamp(value.seconds)
        .and_then(|date| date.replace_nanosecond(value.nanos as u32))
        .map(Timestamp::from_datetime)
        .map_err(|_| Status::invalid_argument("invalid timestamp"))
}

pub(super) fn timestamp(value: &Timestamp) -> Result<prost_types::Timestamp, Status> {
    let time = time::OffsetDateTime::parse(
        value.as_str(),
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| Status::internal("invalid stored timestamp"))?;
    Ok(prost_types::Timestamp {
        seconds: time.unix_timestamp(),
        nanos: time.nanosecond() as i32,
    })
}

pub(super) fn execution_plugin(
    value: &dokime_app::execution::ExecutionPluginIdentity,
) -> Result<v1::ExecutionPluginIdentity, Status> {
    use dokime_app::execution::ExecutionImplementation;
    use v1::execution_plugin_identity::Implementation;

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

    Ok(v1::ExecutionPluginIdentity {
        source: value.source.clone(),
        version: value.version.clone(),
        implementation: Some(implementation),
    })
}

/* === Private Functions === */
fn elapsed_unit(value: &ElapsedUnit) -> v1::ElapsedUnit {
    match value {
        ElapsedUnit::Seconds => v1::ElapsedUnit::Seconds,
        ElapsedUnit::Milliseconds => v1::ElapsedUnit::Milliseconds,
        ElapsedUnit::Minutes => v1::ElapsedUnit::Minutes,
    }
}

fn datetime_timestamp(value: time::OffsetDateTime) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: value.unix_timestamp(),
        nanos: value.nanosecond() as i32,
    }
}

/* === Tests === */

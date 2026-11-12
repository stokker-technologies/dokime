/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Data

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;

// Third Party Imports
use prost_reflect::{DynamicMessage, Kind, ReflectMessage, Value as DynamicValue};

// Local Imports
use crate::{Error, Message, Result, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// A resolved parameter scalar. Integer conversions refuse overflow and mixed types.
pub trait FromValue: Sized {
    fn from_value(value: &v1::Value) -> Result<Self>;
}

/* === Implementations === */

impl FromValue for bool {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::Boolean(value)) => Ok(value),
            _ => Err(invalid("Expected Boolean")),
        }
    }
}

impl FromValue for String {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match &value.kind {
            Some(v1::value::Kind::Text(value)) => Ok(value.clone()),
            _ => Err(invalid("Expected Text")),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::Number(value)) if value.is_finite() => Ok(value),
            _ => Err(invalid("Expected Finite Number")),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::Integer(value)) => Ok(value),
            _ => Err(invalid("Expected Signed Integer")),
        }
    }
}

impl FromValue for u64 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::UnsignedInteger(value)) => Ok(value),
            _ => Err(invalid("Expected Unsigned Integer")),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match &value.kind {
            Some(v1::value::Kind::List(list)) => list.values.iter().map(T::from_value).collect(),
            _ => Err(invalid("Expected List")),
        }
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::Null(_)) => Ok(None),
            _ => T::from_value(value).map(Some),
        }
    }
}

impl FromValue for i32 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        i32::try_from(i64::from_value(value)?).map_err(invalid)
    }
}

impl FromValue for u32 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        u32::try_from(u64::from_value(value)?).map_err(invalid)
    }
}

impl FromValue for f32 {
    fn from_value(value: &v1::Value) -> Result<Self> {
        let number = f64::from_value(value)? as f32;
        if !number.is_finite() {
            return Err(invalid("Number Exceeds f32 Range"));
        }
        Ok(number)
    }
}

impl FromValue for prost_types::Timestamp {
    fn from_value(value: &v1::Value) -> Result<Self> {
        match value.kind {
            Some(v1::value::Kind::Datetime(value)) => Ok(value),
            _ => Err(invalid("Expected Timestamp")),
        }
    }
}

/* === Public Functions === */

pub(crate) fn decode<T: Message>(values: &HashMap<String, v1::Value>) -> Result<T> {
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(T::types()).map_err(invalid)?;
    let descriptor = pool.get_message_by_name(&T::name()).ok_or_else(|| invalid("Missing Data Descriptor"))?;
    let mut message = DynamicMessage::new(descriptor.clone());
    for identifier in values.keys() {
        if descriptor.get_field_by_name(identifier).is_none() {
            return Err(invalid(format!("Unknown Data Field: {identifier}")));
        }
    }
    for field in descriptor.fields() {
        let Some(value) = values.get(field.name()) else {
            if field.supports_presence() || field.is_list() {
                continue;
            }
            return Err(invalid(format!("Missing Data Field: {}", field.name())));
        };
        if matches!(value.kind, Some(v1::value::Kind::Null(_))) && field.supports_presence() {
            continue;
        }
        let value = if field.is_list() {
            let Some(v1::value::Kind::List(list)) = &value.kind else {
                return Err(invalid(format!("Expected List: {}", field.name())));
            };
            DynamicValue::List(list.values.iter().map(|value| dynamic(&field.kind(), value)).collect::<Result<_>>()?)
        } else {
            dynamic(&field.kind(), value)?
        };
        message.try_set_field(&field, value).map_err(invalid)?;
    }
    T::decode(prost::Message::encode_to_vec(&message).as_slice()).map_err(invalid)
}

pub(crate) fn parameters<T: Message>(defaults: T) -> Result<Vec<v1::Parameter>> {
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(T::types()).map_err(invalid)?;
    let descriptor = pool.get_message_by_name(&T::name()).ok_or_else(|| invalid("Missing Parameter Descriptor"))?;
    let message = DynamicMessage::decode(descriptor.clone(), defaults.encode_to_vec().as_slice()).map_err(invalid)?;
    descriptor
        .fields()
        .map(|field| {
            if field.is_map() {
                return Err(invalid(format!("Parameter {} Cannot Be a Map", field.name())));
            }
            let element_type = value_type(&field.kind())?;
            let value = if field.is_list() {
                let value = message.get_field(&field);
                let DynamicValue::List(list) = value.as_ref() else {
                    return Err(invalid("Invalid List Default"));
                };
                v1::Value {
                    kind: Some(v1::value::Kind::List(v1::ValueList {
                        values: list.iter().map(scalar).collect::<Result<_>>()?,
                    })),
                }
            } else if field.supports_presence() && !message.has_field(&field) {
                v1::Value {
                    kind: Some(v1::value::Kind::Null(v1::Null {})),
                }
            } else {
                scalar(message.get_field(&field).as_ref())?
            };
            Ok(v1::Parameter {
                metadata: Some(v1::Metadata {
                    identifier: field.name().into(),
                    label: field
                        .name()
                        .split('_')
                        .map(|word| {
                            let mut letters = word.chars();
                            match letters.next() {
                                Some(first) => first.to_uppercase().to_string() + letters.as_str(),
                                None => String::new(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                    description: String::new(),
                }),
                r#type: if field.is_list() { v1::ValueType::List as i32 } else { element_type as i32 },
                element_type: if field.is_list() { element_type as i32 } else { 0 },
                expression: Some(v1::ParameterExpression {
                    kind: Some(v1::parameter_expression::Kind::Literal(value)),
                }),
                ..Default::default()
            })
        })
        .collect()
}

pub(crate) fn settings<T: Message>(defaults: T) -> Result<Vec<v1::Setting>> {
    parameters(defaults)?
        .into_iter()
        .map(|parameter| {
            let value = parameter.expression.and_then(|expression| match expression.kind {
                Some(v1::parameter_expression::Kind::Literal(value)) => value.kind,
                _ => None,
            });
            let kind = match value {
                Some(v1::value::Kind::Boolean(value)) => v1::setting::Kind::Boolean(v1::BooleanSetting { default_value: value }),
                Some(v1::value::Kind::Text(value)) => v1::setting::Kind::Text(v1::TextSetting { default_value: value, required: false }),
                Some(v1::value::Kind::Number(value)) => v1::setting::Kind::Number(v1::NumberSetting {
                    default_value: value,
                    ..Default::default()
                }),
                _ => return Err(invalid("Setting Defaults Require Boolean, Text or f64 Numbers; Use an Explicit Setting for Other Kinds")),
            };
            Ok(v1::Setting {
                metadata: parameter.metadata,
                kind: Some(kind),
            })
        })
        .collect()
}

/* === Private Functions === */

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}

fn value_type(kind: &Kind) -> Result<v1::ValueType> {
    Ok(match kind {
        Kind::Bool => v1::ValueType::Boolean,
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => v1::ValueType::Integer,
        Kind::Uint32 | Kind::Fixed32 | Kind::Uint64 | Kind::Fixed64 => v1::ValueType::UnsignedInteger,
        Kind::Float | Kind::Double => v1::ValueType::Number,
        Kind::String => v1::ValueType::Text,
        Kind::Bytes => v1::ValueType::Bytes,
        Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => v1::ValueType::Datetime,
        _ => return Err(invalid("Parameters and Settings Require Scalar or List Data")),
    })
}

fn scalar(value: &DynamicValue) -> Result<v1::Value> {
    use v1::value::Kind as V;
    let kind = match value {
        DynamicValue::Bool(value) => V::Boolean(*value),
        DynamicValue::I32(value) => V::Integer(i64::from(*value)),
        DynamicValue::I64(value) => V::Integer(*value),
        DynamicValue::U32(value) => V::UnsignedInteger(u64::from(*value)),
        DynamicValue::U64(value) => V::UnsignedInteger(*value),
        DynamicValue::F32(value) if value.is_finite() => V::Number(f64::from(*value)),
        DynamicValue::F64(value) if value.is_finite() => V::Number(*value),
        DynamicValue::String(value) => V::Text(value.clone()),
        DynamicValue::Bytes(value) => V::Data(value.to_vec()),
        DynamicValue::Message(message) if message.descriptor().full_name() == "google.protobuf.Timestamp" => V::Datetime(
            <prost_types::Timestamp as prost::Message>::decode(prost::Message::encode_to_vec(message).as_slice()).map_err(invalid)?,
        ),
        _ => return Err(invalid("Unsupported or Nonfinite Default Value")),
    };
    Ok(v1::Value { kind: Some(kind) })
}

fn dynamic(kind: &Kind, value: &v1::Value) -> Result<DynamicValue> {
    use v1::value::Kind as V;
    Ok(match (kind, &value.kind) {
        (Kind::Bool, Some(V::Boolean(value))) => DynamicValue::Bool(*value),
        (Kind::String, Some(V::Text(value))) => DynamicValue::String(value.clone()),
        (Kind::Bytes, Some(V::Data(value))) => DynamicValue::Bytes(value.clone().into()),
        (Kind::Double, Some(V::Number(value))) if value.is_finite() => DynamicValue::F64(*value),
        (Kind::Float, Some(V::Number(value))) if value.is_finite() && (*value as f32).is_finite() => DynamicValue::F32(*value as f32),
        (Kind::Int32 | Kind::Sint32 | Kind::Sfixed32, Some(V::Integer(value))) => {
            DynamicValue::I32(i32::try_from(*value).map_err(invalid)?)
        }
        (Kind::Int64 | Kind::Sint64 | Kind::Sfixed64, Some(V::Integer(value))) => DynamicValue::I64(*value),
        (Kind::Uint32 | Kind::Fixed32, Some(V::UnsignedInteger(value))) => DynamicValue::U32(u32::try_from(*value).map_err(invalid)?),
        (Kind::Uint64 | Kind::Fixed64, Some(V::UnsignedInteger(value))) => DynamicValue::U64(*value),
        (Kind::Message(descriptor), Some(V::Datetime(value))) if descriptor.full_name() == "google.protobuf.Timestamp" => {
            DynamicValue::Message(
                DynamicMessage::decode(descriptor.clone(), prost::Message::encode_to_vec(value).as_slice()).map_err(invalid)?,
            )
        }
        _ => return Err(invalid("Data Value Does Not Match Its Declared Type")),
    })
}

/* === Tests === */

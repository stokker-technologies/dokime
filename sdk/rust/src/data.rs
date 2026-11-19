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
use prost_reflect::{DynamicMessage, Kind, ReflectMessage};

// Local Imports
use crate::ExactNumber;
use crate::{Error, Message, Result};
use crate::{
    Metadata, Null, Parameter, ParameterExpression, SettingDeclaration, SettingValue, SettingValues, Value, ValueList, ValueType,
    parameter_expression, setting_value, value,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// A resolved parameter scalar. Integer conversions refuse overflow and mixed types.
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self>;
}

/* === Implementations === */

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::Boolean(value)) => Ok(value),
            _ => Err(invalid("Expected Boolean")),
        }
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self> {
        match &value.kind {
            Some(value::Kind::Text(value)) => Ok(value.clone()),
            _ => Err(invalid("Expected Text")),
        }
    }
}

impl FromValue for ExactNumber {
    fn from_value(value: &Value) -> Result<Self> {
        match &value.kind {
            Some(value::Kind::Exact(value)) => Ok(value.clone()),
            _ => Err(invalid("Read Exact Number: Expected an Exact Value")),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::Number(value)) if value.is_finite() => Ok(value),
            _ => Err(invalid("Expected Finite Number")),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::Integer(value)) => Ok(value),
            _ => Err(invalid("Expected Signed Integer")),
        }
    }
}

impl FromValue for u64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::UnsignedInteger(value)) => Ok(value),
            _ => Err(invalid("Expected Unsigned Integer")),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: &Value) -> Result<Self> {
        match &value.kind {
            Some(value::Kind::List(list)) => list.values.iter().map(T::from_value).collect(),
            _ => Err(invalid("Expected List")),
        }
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::Null(_)) => Ok(None),
            _ => T::from_value(value).map(Some),
        }
    }
}

impl FromValue for i32 {
    fn from_value(value: &Value) -> Result<Self> {
        i32::try_from(i64::from_value(value)?).map_err(invalid)
    }
}

impl FromValue for u32 {
    fn from_value(value: &Value) -> Result<Self> {
        u32::try_from(u64::from_value(value)?).map_err(invalid)
    }
}

impl FromValue for f32 {
    fn from_value(value: &Value) -> Result<Self> {
        let number = f64::from_value(value)? as f32;
        if !number.is_finite() {
            return Err(invalid("Number Exceeds f32 Range"));
        }
        Ok(number)
    }
}

impl FromValue for prost_types::Timestamp {
    fn from_value(value: &Value) -> Result<Self> {
        match value.kind {
            Some(value::Kind::Datetime(value)) => Ok(value),
            _ => Err(invalid("Expected Timestamp")),
        }
    }
}

/* === Public Functions === */

pub(crate) fn decode<T: Message>(values: &HashMap<String, Value>) -> Result<T> {
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
        if matches!(value.kind, Some(value::Kind::Null(_))) && field.supports_presence() {
            continue;
        }
        let value = if field.is_list() {
            let Some(value::Kind::List(list)) = &value.kind else {
                return Err(invalid(format!("Expected List: {}", field.name())));
            };
            prost_reflect::Value::List(list.values.iter().map(|value| dynamic(&field.kind(), value)).collect::<Result<_>>()?)
        } else {
            dynamic(&field.kind(), value)?
        };
        message.try_set_field(&field, value).map_err(invalid)?;
    }
    T::decode(prost::Message::encode_to_vec(&message).as_slice()).map_err(invalid)
}

pub(crate) fn parameters<T: Message>(defaults: T) -> Result<Vec<Parameter>> {
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
                let prost_reflect::Value::List(list) = value.as_ref() else {
                    return Err(invalid("Invalid List Default"));
                };
                Value {
                    kind: Some(value::Kind::List(ValueList {
                        values: list.iter().map(scalar).collect::<Result<_>>()?,
                    })),
                }
            } else if field.supports_presence() && !message.has_field(&field) {
                Value { kind: Some(value::Kind::Null(Null {})) }
            } else {
                scalar(message.get_field(&field).as_ref())?
            };
            Ok(Parameter {
                metadata: Some(Metadata {
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
                    ..Default::default()
                }),
                r#type: if field.is_list() { ValueType::List as i32 } else { element_type as i32 },
                element_type: if field.is_list() { element_type as i32 } else { 0 },
                expression: Some(ParameterExpression {
                    kind: Some(parameter_expression::Kind::Literal(value)),
                }),
                ..Default::default()
            })
        })
        .collect()
}

pub(crate) fn settings<T: Message>(defaults: T) -> Result<Vec<SettingDeclaration>> {
    parameters(defaults)?
        .into_iter()
        .map(|parameter| {
            let metadata = parameter.metadata.ok_or_else(|| invalid("Declare Settings: Missing Metadata"))?;
            let value = parameter.expression.and_then(|expression| match expression.kind {
                Some(parameter_expression::Kind::Literal(value)) => value.kind,
                _ => None,
            });
            let value = match value {
                Some(value::Kind::Boolean(value)) => setting_value::Value::Flag(value),
                Some(value::Kind::Integer(value)) => setting_value::Value::Integer(value),
                Some(value::Kind::Number(value)) => setting_value::Value::Number(value),
                Some(value::Kind::Text(value)) => setting_value::Value::Text(value),
                _ => return Err(invalid("Declare Settings: Supply a Boolean, Signed Integer, Number or Text Default")),
            };
            Ok(SettingDeclaration {
                name: metadata.identifier,
                title: metadata.label,
                description: metadata.description,
                default: Some(SettingValue { value: Some(value) }),
            })
        })
        .collect()
}

pub(crate) fn decode_settings<T: Message>(settings: SettingValues) -> Result<T> {
    let values = settings
        .values
        .into_iter()
        .map(|(name, value)| {
            let kind = match value.value {
                Some(setting_value::Value::Flag(value)) => value::Kind::Boolean(value),
                Some(setting_value::Value::Integer(value)) => value::Kind::Integer(value),
                Some(setting_value::Value::Number(value)) => value::Kind::Number(value),
                Some(setting_value::Value::Text(value)) => value::Kind::Text(value),
                _ => return Err(invalid("Read Settings: Unsupported or Missing Typed Value")),
            };
            Ok((name, Value { kind: Some(kind) }))
        })
        .collect::<Result<HashMap<_, _>>>()?;
    decode(&values)
}

/* === Private Functions === */

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}

fn value_type(kind: &Kind) -> Result<ValueType> {
    Ok(match kind {
        Kind::Bool => ValueType::Boolean,
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => ValueType::Integer,
        Kind::Uint32 | Kind::Fixed32 | Kind::Uint64 | Kind::Fixed64 => ValueType::UnsignedInteger,
        Kind::Float | Kind::Double => ValueType::Number,
        Kind::String => ValueType::Text,
        Kind::Bytes => ValueType::Bytes,
        Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => ValueType::Datetime,
        _ => return Err(invalid("Parameters and Settings Require Scalar or List Data")),
    })
}

pub(crate) fn scalar(value: &prost_reflect::Value) -> Result<Value> {
    let kind = match value {
        prost_reflect::Value::Bool(value) => value::Kind::Boolean(*value),
        prost_reflect::Value::I32(value) => value::Kind::Integer(i64::from(*value)),
        prost_reflect::Value::I64(value) => value::Kind::Integer(*value),
        prost_reflect::Value::U32(value) => value::Kind::UnsignedInteger(u64::from(*value)),
        prost_reflect::Value::U64(value) => value::Kind::UnsignedInteger(*value),
        prost_reflect::Value::F32(value) if value.is_finite() => value::Kind::Number(f64::from(*value)),
        prost_reflect::Value::F64(value) if value.is_finite() => value::Kind::Number(*value),
        prost_reflect::Value::String(value) => value::Kind::Text(value.clone()),
        prost_reflect::Value::Bytes(value) => value::Kind::Data(value.to_vec()),
        prost_reflect::Value::Message(message) if message.descriptor().full_name() == "google.protobuf.Timestamp" => value::Kind::Datetime(
            <prost_types::Timestamp as prost::Message>::decode(prost::Message::encode_to_vec(message).as_slice()).map_err(invalid)?,
        ),
        _ => return Err(invalid("Unsupported or Nonfinite Default Value")),
    };
    Ok(Value { kind: Some(kind) })
}

fn dynamic(kind: &Kind, value: &Value) -> Result<prost_reflect::Value> {
    Ok(match (kind, &value.kind) {
        (Kind::Bool, Some(value::Kind::Boolean(value))) => prost_reflect::Value::Bool(*value),
        (Kind::String, Some(value::Kind::Text(value))) => prost_reflect::Value::String(value.clone()),
        (Kind::Bytes, Some(value::Kind::Data(value))) => prost_reflect::Value::Bytes(value.clone().into()),
        (Kind::Double, Some(value::Kind::Number(value))) if value.is_finite() => prost_reflect::Value::F64(*value),
        (Kind::Float, Some(value::Kind::Number(value))) if value.is_finite() && (*value as f32).is_finite() => {
            prost_reflect::Value::F32(*value as f32)
        }
        (Kind::Int32 | Kind::Sint32 | Kind::Sfixed32, Some(value::Kind::Integer(value))) => {
            prost_reflect::Value::I32(i32::try_from(*value).map_err(invalid)?)
        }
        (Kind::Int64 | Kind::Sint64 | Kind::Sfixed64, Some(value::Kind::Integer(value))) => prost_reflect::Value::I64(*value),
        (Kind::Uint32 | Kind::Fixed32, Some(value::Kind::UnsignedInteger(value))) => {
            prost_reflect::Value::U32(u32::try_from(*value).map_err(invalid)?)
        }
        (Kind::Uint64 | Kind::Fixed64, Some(value::Kind::UnsignedInteger(value))) => prost_reflect::Value::U64(*value),
        (Kind::Message(descriptor), Some(value::Kind::Datetime(value))) if descriptor.full_name() == "google.protobuf.Timestamp" => {
            prost_reflect::Value::Message(
                DynamicMessage::decode(descriptor.clone(), prost::Message::encode_to_vec(value).as_slice()).map_err(invalid)?,
            )
        }
        _ => return Err(invalid("Data Value Does Not Match Its Declared Type")),
    })
}

/* === Tests === */

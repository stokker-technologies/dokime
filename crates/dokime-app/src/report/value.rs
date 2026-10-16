/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod serialization;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, fmt};

// Third Party Imports
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, SeqAccess, Visitor},
};

// Local Imports
use crate::{Error, Result};

/* === Constants === */
const MAX_DEPTH: usize = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

/* === Traits === */

/* === Implementations === */
impl Value {
    pub fn from_serializable<T: ?Sized + Serialize>(source: &T) -> Result<Self> {
        serialization::build(source, std::num::NonZeroUsize::MAX)
    }

    pub(super) fn memory_at(&self, depth: usize) -> usize {
        if depth > MAX_DEPTH {
            return usize::MAX;
        }
        let heap = match self {
            Self::Null | Self::Bool(_) | Self::Integer(_) | Self::Unsigned(_) | Self::Number(_) => {
                0
            }
            Self::String(text) => text.capacity(),
            Self::Array(values) => values.iter().fold(
                values
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Self>()),
                |bytes, value| {
                    excess(
                        bytes,
                        value.memory_at(depth + 1),
                        std::mem::size_of::<Self>(),
                    )
                },
            ),
            Self::Object(values) => {
                // An Empty Map May Retain Its Original Root After Removal.
                let nodes = harmos::btree_map_memory::<String, Self>(values.len().max(1));
                if nodes == usize::MAX {
                    return usize::MAX;
                }
                values.iter().fold(
                    nodes.saturating_sub(std::mem::size_of::<BTreeMap<String, Self>>()),
                    |bytes, (key, value)| {
                        excess(
                            bytes.saturating_add(key.capacity()),
                            value.memory_at(depth + 1),
                            std::mem::size_of::<Self>(),
                        )
                    },
                )
            }
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}
// Dynamic JSON Keeps Its Existing Numeric Equality Across Import Paths.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Unsigned(left), Self::Unsigned(right)) => left == right,
            (Self::Integer(left), Self::Unsigned(right))
            | (Self::Unsigned(right), Self::Integer(left)) => {
                u64::try_from(*left).ok() == Some(*right)
            }
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => left == right,
            (Self::Object(left), Self::Object(right)) => left == right,
            _ => false,
        }
    }
}

impl harmos::MemorySize for Value {
    fn memory_size(&self) -> usize {
        self.memory_at(0)
    }
}
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Unsigned(value) => serializer.serialize_u64(*value),
            Self::Number(value) if value.is_finite() => serializer.serialize_f64(*value),
            Self::Number(_) => Err(serde::ser::Error::custom("report number must be finite")),
            Self::String(value) => serializer.serialize_str(value),
            Self::Array(values) => values.serialize(serializer),
            Self::Object(values) => values.serialize(serializer),
        }
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct Owned;
        impl<'de> Visitor<'de> for Owned {
            type Value = Value;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a report JSON value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> std::result::Result<Value, E> {
                Ok(Value::Bool(value))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> std::result::Result<Value, E> {
                Ok(Value::Integer(value))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> std::result::Result<Value, E> {
                Ok(Value::Unsigned(value))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> std::result::Result<Value, E> {
                if !value.is_finite() {
                    return Err(E::custom("report number must be finite"));
                }
                Ok(Value::Number(value))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Value, E> {
                Ok(Value::String(value.to_owned()))
            }
            fn visit_string<E: serde::de::Error>(
                self,
                value: String,
            ) -> std::result::Result<Value, E> {
                Ok(Value::String(value))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    if values.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("report object key is duplicated"));
                    }
                }
                Ok(Value::Object(values))
            }
        }
        deserializer.deserialize_any(Owned)
    }
}
impl TryFrom<serde_json::Value> for Value {
    type Error = Error;
    fn try_from(value: serde_json::Value) -> Result<Self> {
        from_json(value, 0)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn excess(bytes: usize, owned: usize, inline: usize) -> usize {
    if owned == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_add(owned.saturating_sub(inline))
    }
}
// Standalone JSON Imports Only; Native Capture Producers Construct Owned Values Directly.
fn from_json(value: serde_json::Value, depth: usize) -> Result<Value> {
    if depth > MAX_DEPTH {
        return Err(Error::InvalidReportInputs(
            "report value nesting exceeds limit".into(),
        ));
    }
    Ok(match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_u64() {
                Value::Unsigned(value)
            } else if let Some(value) = value.as_f64().filter(|value| value.is_finite()) {
                Value::Number(value)
            } else {
                return Err(Error::InvalidReportInputs(
                    "report number is not a finite native scalar".into(),
                ));
            }
        }
        serde_json::Value::String(value) => Value::String(value),
        serde_json::Value::Array(values) => {
            let mut owned = Vec::new();
            owned
                .try_reserve_exact(values.len())
                .map_err(|_| Error::InvalidReportInputs("report array allocation failed".into()))?;
            for value in values {
                owned.push(from_json(value, depth + 1)?);
            }
            Value::Array(owned)
        }
        serde_json::Value::Object(values) => {
            let mut owned = BTreeMap::new();
            for (key, value) in values {
                owned.insert(key, from_json(value, depth + 1)?);
            }
            Value::Object(owned)
        }
    })
}

/* === Tests === */

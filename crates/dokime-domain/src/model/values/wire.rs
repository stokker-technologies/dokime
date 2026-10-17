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
use std::collections::BTreeMap;

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

// Local Imports
use crate::model::identity::wire::Timestamp;
use crate::model::values::Value as Core;

/* === Constants === */
// Nesting deeper than this is refused when a wire value is rebuilt.
pub const MAX_DEPTH: usize = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A dynamic value flattened into its nodes, so its schema stays small and its
// nesting is bounded by data rather than by the protobuf descriptor.
//
// `nodes[0]` is the root. An array or object names its members by their
// index in `nodes`; every index exceeds its container's index and every node
// other than the root is a member of exactly one container.
#[harmos::message]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Value {
    pub nodes: Vec<ValueNode>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueNode {
    pub kind: Option<ValueKind>,
}

// Member indices of an array, in order.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueItems {
    pub items: Vec<u32>,
}

// Members of an object, in key order.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueFields {
    pub fields: Vec<ValueField>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueField {
    pub key: String,
    pub value: u32,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueKind {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    // IEEE-754 binary64, including NaN and infinities.
    Number(f64),
    Text(String),
    Datetime(Timestamp),
    Array(ValueItems),
    Object(ValueFields),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ValueError {
    #[error("value has no root node")]
    Empty,
    #[error("value node {0} has no kind")]
    Unset(usize),
    #[error("value member index {0} is out of order or out of range")]
    Member(usize),
    #[error("value node {0} is not a member of exactly one container")]
    Shared(usize),
    #[error("value nests deeper than {MAX_DEPTH} levels")]
    Depth,
    #[error("value timestamp is out of range")]
    Timestamp,
    #[error("value object repeats a key")]
    Duplicate,
}

/* === Traits === */

/* === Implementations === */
impl Value {
    pub fn null() -> Self {
        Self::from(&crate::model::values::Value::Null)
    }

    pub fn to_core(&self) -> Result<crate::model::values::Value, ValueError> {
        if self.nodes.is_empty() {
            return Err(ValueError::Empty);
        }
        let mut members = vec![false; self.nodes.len()];
        let value = self.rebuild(0, 0, &mut members)?;
        if let Some(index) = members.iter().skip(1).position(|member| !member) {
            return Err(ValueError::Shared(index + 1));
        }
        Ok(value)
    }

    fn rebuild(
        &self,
        index: usize,
        depth: usize,
        members: &mut [bool],
    ) -> Result<crate::model::values::Value, ValueError> {
        use crate::model::values::Value as Core;
        if depth > MAX_DEPTH {
            return Err(ValueError::Depth);
        }
        let member = |child: u32, members: &mut [bool]| {
            let child = child as usize;
            if child <= index || child >= self.nodes.len() {
                return Err(ValueError::Member(index));
            }
            if std::mem::replace(&mut members[child], true) {
                return Err(ValueError::Shared(child));
            }
            Ok(child)
        };
        let kind = self.nodes[index]
            .kind
            .as_ref()
            .ok_or(ValueError::Unset(index))?;
        Ok(match kind {
            ValueKind::Null => Core::Null,
            ValueKind::Bool(value) => Core::Bool(*value),
            ValueKind::Integer(value) => Core::Integer(*value),
            ValueKind::Unsigned(value) => Core::Unsigned(*value),
            ValueKind::Number(value) => Core::Number(*value),
            ValueKind::Text(value) => Core::Text(value.clone()),
            ValueKind::Datetime(value) => {
                Core::Datetime(value.to_datetime().map_err(|_| ValueError::Timestamp)?)
            }
            ValueKind::Array(items) => {
                let mut values = Vec::with_capacity(items.items.len());
                for child in &items.items {
                    let child = member(*child, members)?;
                    values.push(self.rebuild(child, depth + 1, members)?);
                }
                Core::Array(values)
            }
            ValueKind::Object(fields) => {
                let mut entries = BTreeMap::new();
                for field in &fields.fields {
                    let child = member(field.value, members)?;
                    let value = self.rebuild(child, depth + 1, members)?;
                    if entries.insert(field.key.clone(), value).is_some() {
                        return Err(ValueError::Duplicate);
                    }
                }
                Core::Object(entries)
            }
        })
    }

    fn push(&mut self, value: &crate::model::values::Value) -> u32 {
        let index = self.nodes.len();
        self.nodes.push(ValueNode { kind: None });
        let kind = match value {
            Core::Null => ValueKind::Null,
            Core::Bool(value) => ValueKind::Bool(*value),
            Core::Integer(value) => ValueKind::Integer(*value),
            Core::Unsigned(value) => ValueKind::Unsigned(*value),
            Core::Number(value) => ValueKind::Number(*value),
            Core::Text(value) => ValueKind::Text(value.clone()),
            Core::Datetime(value) => ValueKind::Datetime(Timestamp::from(*value)),
            Core::Array(values) => ValueKind::Array(ValueItems {
                items: values.iter().map(|value| self.push(value)).collect(),
            }),
            Core::Object(entries) => ValueKind::Object(ValueFields {
                fields: entries
                    .iter()
                    .map(|(key, value)| ValueField {
                        key: key.clone(),
                        value: self.push(value),
                    })
                    .collect(),
            }),
        };
        self.nodes[index].kind = Some(kind);
        u32::try_from(index).expect("a value never exceeds u32 nodes")
    }
}

impl From<&crate::model::values::Value> for Value {
    fn from(value: &crate::model::values::Value) -> Self {
        let mut wire = Self::default();
        wire.push(value);
        wire
    }
}

impl From<&BTreeMap<String, crate::model::values::Value>> for Value {
    fn from(entries: &BTreeMap<String, crate::model::values::Value>) -> Self {
        let mut wire = Self::default();
        wire.nodes.push(ValueNode { kind: None });
        let fields = entries
            .iter()
            .map(|(key, value)| ValueField {
                key: key.clone(),
                value: wire.push(value),
            })
            .collect();
        wire.nodes[0].kind = Some(ValueKind::Object(ValueFields { fields }));
        wire
    }
}

impl From<crate::model::values::Value> for Value {
    fn from(value: crate::model::values::Value) -> Self {
        Self::from(&value)
    }
}

impl TryFrom<&Value> for crate::model::values::Value {
    type Error = ValueError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        value.to_core()
    }
}

impl From<OffsetDateTime> for Timestamp {
    fn from(value: OffsetDateTime) -> Self {
        Self {
            seconds: value.unix_timestamp(),
            nanos: value.nanosecond() as i32,
        }
    }
}

impl Timestamp {
    pub fn to_datetime(self) -> Result<OffsetDateTime, time::error::ComponentRange> {
        OffsetDateTime::from_unix_timestamp(self.seconds)?.replace_nanosecond(self.nanos as u32)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

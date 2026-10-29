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

// Local Imports
use crate::model::time::Timestamp;
use crate::model::values::Value;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */

product_error!(FieldError, formatter {
    Self { field, declared, found: Some(found) } => write!(
        formatter,
        "field {field:?} was declared {declared:?} and the evidence carries {found}"
    ),
    Self { field, declared, found: None } => write!(
        formatter,
        "field {field:?} was declared {declared:?} and the evidence carries nothing"
    ),
});

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    pub field: String,
    pub declared: ValueType,
    pub found: Option<String>,
}

/* === Enums === */

/* === Traits === */

pub trait FieldValue: Sized {
    const VALUE_TYPE: ValueType;

    fn into_value(self) -> Value;

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError>;
}

/* === Implementations === */

impl FieldError {
    #[must_use]
    pub fn named(mut self, field: impl Into<String>) -> Self {
        self.field = field.into();
        self
    }

    fn of(declared: ValueType, found: Option<&Value>) -> Self {
        Self {
            field: String::new(),
            declared,
            found: found.map(described),
        }
    }
}

impl FieldValue for bool {
    const VALUE_TYPE: ValueType = ValueType::Boolean;

    fn into_value(self) -> Value {
        Value::Bool(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Bool(boolean)) => Ok(*boolean),
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl FieldValue for i64 {
    const VALUE_TYPE: ValueType = ValueType::Integer;

    fn into_value(self) -> Value {
        Value::Integer(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Integer(integer)) => Ok(*integer),
            Some(Value::Unsigned(integer)) => i64::try_from(*integer).map_err(|_| FieldError::of(ValueType::Integer, value)),
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl FieldValue for i32 {
    const VALUE_TYPE: ValueType = ValueType::Integer;

    fn into_value(self) -> Value {
        Value::Integer(i64::from(self))
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        narrowed(value).and_then(|integer| Self::try_from(integer).map_err(|_| FieldError::of(Self::VALUE_TYPE, value)))
    }
}

impl FieldValue for u32 {
    const VALUE_TYPE: ValueType = ValueType::Integer;

    fn into_value(self) -> Value {
        Value::Integer(i64::from(self))
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        narrowed(value).and_then(|integer| Self::try_from(integer).map_err(|_| FieldError::of(Self::VALUE_TYPE, value)))
    }
}

impl FieldValue for u16 {
    const VALUE_TYPE: ValueType = ValueType::Integer;

    fn into_value(self) -> Value {
        Value::Integer(i64::from(self))
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        narrowed(value).and_then(|integer| Self::try_from(integer).map_err(|_| FieldError::of(Self::VALUE_TYPE, value)))
    }
}

impl FieldValue for u64 {
    const VALUE_TYPE: ValueType = ValueType::Integer;

    fn into_value(self) -> Value {
        Value::Unsigned(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Unsigned(integer)) => Ok(*integer),
            other => narrowed(other).and_then(|integer| Self::try_from(integer).map_err(|_| FieldError::of(Self::VALUE_TYPE, value))),
        }
    }
}

impl FieldValue for f64 {
    const VALUE_TYPE: ValueType = ValueType::Number;

    fn into_value(self) -> Value {
        Value::Number(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Number(number)) => Ok(*number),
            Some(integer @ (Value::Integer(_) | Value::Unsigned(_))) => {
                let number = crate::model::values::numeric_value(integer).expect("integer projection");
                if crate::model::values::numeric_cmp(integer, &Value::Number(number)) == Some(std::cmp::Ordering::Equal) {
                    Ok(number)
                } else {
                    Err(FieldError::of(Self::VALUE_TYPE, value))
                }
            }
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl FieldValue for f32 {
    const VALUE_TYPE: ValueType = ValueType::Number;

    fn into_value(self) -> Value {
        Value::Number(f64::from(self))
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        f64::from_value(value).map(|number| number as Self)
    }
}

impl FieldValue for String {
    const VALUE_TYPE: ValueType = ValueType::Text;

    fn into_value(self) -> Value {
        Value::Text(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Text(text)) => Ok(text.clone()),
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl FieldValue for Timestamp {
    const VALUE_TYPE: ValueType = ValueType::Datetime;

    fn into_value(self) -> Value {
        Value::from(self)
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Datetime(timestamp)) => Ok(Timestamp::from_datetime(*timestamp)),
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl<T: FieldValue> FieldValue for Vec<T> {
    const VALUE_TYPE: ValueType = ValueType::Array;

    fn into_value(self) -> Value {
        Value::Array(self.into_iter().map(T::into_value).collect())
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            Some(Value::Array(elements)) => elements.iter().map(|element| T::from_value(Some(element))).collect(),
            other => Err(FieldError::of(Self::VALUE_TYPE, other)),
        }
    }
}

impl<T: FieldValue> FieldValue for Option<T> {
    const VALUE_TYPE: ValueType = T::VALUE_TYPE;

    fn into_value(self) -> Value {
        match self {
            Some(value) => value.into_value(),
            None => Value::Null,
        }
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        match value {
            None | Some(Value::Null) => Ok(None),
            present => T::from_value(present).map(Some),
        }
    }
}

/* === Public Functions === */

pub fn field<T: FieldValue>(values: &BTreeMap<String, Value>, name: &str) -> Result<T, FieldError> {
    T::from_value(values.get(name)).map_err(|error| error.named(name))
}

/* === Private Functions === */

fn described(value: &Value) -> String {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Integer(_) | Value::Unsigned(_) => "an integer",
        Value::Number(_) => "a number",
        Value::Text(_) => "text",
        Value::Datetime(_) => "a datetime",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
    .to_owned()
}

fn narrowed(value: Option<&Value>) -> Result<i64, FieldError> {
    match value {
        Some(Value::Integer(integer)) => Ok(*integer),
        Some(Value::Unsigned(integer)) => i64::try_from(*integer).map_err(|_| FieldError::of(ValueType::Integer, value)),
        other => Err(FieldError::of(ValueType::Integer, other)),
    }
}

/* === Tests === */

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
pub use harmos::Value;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    Boolean,
    Integer,
    Number,
    Text,
    Datetime,
    File,
    Object,
    Array,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn value_matches_type(value: &Value, value_type: &ValueType) -> bool {
    match value_type {
        ValueType::Boolean => matches!(value, Value::Bool(_)),
        ValueType::Integer => matches!(value, Value::Integer(_) | Value::Unsigned(_)),
        ValueType::Number => match value {
            Value::Integer(_) | Value::Unsigned(_) => true,
            Value::Number(value) => value.is_finite(),
            _ => false,
        },
        ValueType::Text | ValueType::File => matches!(value, Value::Text(_)),
        ValueType::Datetime => matches!(value, Value::Datetime(_)),
        ValueType::Object => matches!(value, Value::Object(_)),
        ValueType::Array => matches!(value, Value::Array(_)),
    }
}

pub fn datetime_seconds(value: &Value) -> Option<f64> {
    match value {
        Value::Datetime(timestamp) => {
            Some(timestamp.unix_timestamp_nanos() as f64 / 1_000_000_000.0)
        }
        _ => None,
    }
}

pub fn numeric_value(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Unsigned(value) => Some(*value as f64),
        Value::Number(value) if value.is_finite() => Some(*value),
        _ => None,
    }
}

pub fn numeric_cmp(value: &Value, bound: &Value) -> Option<std::cmp::Ordering> {
    match (value, bound) {
        (Value::Integer(value), Value::Integer(bound)) => Some(value.cmp(bound)),
        (Value::Unsigned(value), Value::Unsigned(bound)) => Some(value.cmp(bound)),
        (Value::Integer(value), Value::Unsigned(bound)) => {
            Some(i128::from(*value).cmp(&i128::from(*bound)))
        }
        (Value::Unsigned(value), Value::Integer(bound)) => {
            Some(i128::from(*value).cmp(&i128::from(*bound)))
        }
        (Value::Unsigned(value), Value::Number(bound)) => unsigned_float_cmp(*value, *bound),
        (Value::Number(value), Value::Unsigned(bound)) => {
            unsigned_float_cmp(*bound, *value).map(std::cmp::Ordering::reverse)
        }
        (Value::Number(value), Value::Number(bound)) if value.is_finite() && bound.is_finite() => {
            value.partial_cmp(bound)
        }
        (Value::Integer(value), Value::Number(bound)) => integer_float_cmp(*value, *bound),
        (Value::Number(value), Value::Integer(bound)) => {
            integer_float_cmp(*bound, *value).map(std::cmp::Ordering::reverse)
        }
        _ => None,
    }
}

/* === Private Functions === */
fn integer_float_cmp(value: i64, bound: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if !bound.is_finite() {
        return None;
    }
    if bound < i64::MIN as f64 {
        return Some(Ordering::Greater);
    }
    // i64::MAX rounds up to exactly 2^63 as f64.
    if bound >= i64::MAX as f64 {
        return Some(Ordering::Less);
    }
    match value.cmp(&(bound as i64)) {
        Ordering::Equal => 0.0_f64.partial_cmp(&bound.fract()),
        order => Some(order),
    }
}

fn unsigned_float_cmp(value: u64, bound: f64) -> Option<std::cmp::Ordering> {

    if !bound.is_finite() {
        return None;
    }
    if bound < 0.0 {
        return Some(Ordering::Greater);
    }
    if bound >= 18_446_744_073_709_551_616.0 {
        return Some(Ordering::Less);
    }

    match value.cmp(&(bound as u64)) {
        Ordering::Equal => 0.0_f64.partial_cmp(&bound.fract()),
        order => Some(order),
    }
}

/* === Tests === */

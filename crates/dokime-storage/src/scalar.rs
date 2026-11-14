/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Projected Scalar Query Values

/* === Modules === */

/* === Re-Exports === */

pub(crate) use harmos_storage::query::scalar::{Agg, ScalarValue};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray, StructArray, UInt64Array};

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn column<'a>(batch: &'a RecordBatch, path: &str) -> Result<Column<'a>> {
    let values = batch.column_by_name("values").ok_or(Error::Corrupt("values column missing"))?;
    let mut array = values;
    let mut parents = Vec::new();
    for name in path.split('.') {
        let structure = array.as_any().downcast_ref::<StructArray>().ok_or(Error::Data("select a scalar field path"))?;
        parents.push(array);
        array = structure.column_by_name(name).ok_or(Error::Data("declare the selected scalar column"))?;
    }
    Ok(Column { array, parents })
}
pub(crate) struct Column<'a> {
    array: &'a ArrayRef,
    parents: Vec<&'a ArrayRef>,
}
impl Column<'_> {
    pub(crate) fn value(&self, row: usize) -> Result<ScalarValue> {
        if self.array.is_null(row) || self.parents.iter().any(|a| a.is_null(row)) {
            return Ok(ScalarValue::Null);
        }
        let any = self.array.as_any();
        if let Some(a) = any.downcast_ref::<UInt64Array>() {
            return Ok(ScalarValue::Unsigned(a.value(row)));
        }
        if let Some(a) = any.downcast_ref::<Int64Array>() {
            return Ok(ScalarValue::Signed(a.value(row)));
        }
        if let Some(a) = any.downcast_ref::<Float64Array>() {
            return Ok(ScalarValue::Float(a.value(row)));
        }
        if let Some(a) = any.downcast_ref::<StringArray>() {
            return Ok(ScalarValue::Text(a.value(row).into()));
        }
        if let Some(a) = any.downcast_ref::<BooleanArray>() {
            return Ok(ScalarValue::Bool(a.value(row)));
        }
        Err(Error::Data("select a scalar column"))
    }
}
pub(crate) fn field(batch: &RecordBatch, path: &str, row: usize) -> Result<ScalarValue> {
    column(batch, path)?.value(row)
}
pub(crate) fn number(value: &ScalarValue) -> Option<f64> {
    match value {
        ScalarValue::Float(v) => Some(*v),
        ScalarValue::Signed(v) => Some(*v as f64),
        ScalarValue::Unsigned(v) => Some(*v as f64),
        ScalarValue::Exact(v) => Some((if v.negative { -1.0 } else { 1.0 }) * (v.numerator as f64 / v.denominator.get() as f64)),
        ScalarValue::Bool(v) => Some(u8::from(*v) as f64),
        _ => None,
    }
}
pub(crate) fn from_wire(value: &v1::Value) -> Result<ScalarValue> {
    Ok(match value.kind.as_ref() {
        Some(v1::value::Kind::Null(_)) => ScalarValue::Null,
        Some(v1::value::Kind::Number(v)) if v.is_finite() => ScalarValue::Float(*v),
        Some(v1::value::Kind::Integer(v)) => ScalarValue::Signed(*v),
        Some(v1::value::Kind::UnsignedInteger(v)) => ScalarValue::Unsigned(*v),
        Some(v1::value::Kind::Boolean(v)) => ScalarValue::Bool(*v),
        Some(v1::value::Kind::Text(v)) => ScalarValue::Text(v.clone()),
        _ => return Err(Error::Data("select a finite scalar value")),
    })
}
pub(crate) fn wire(value: &ScalarValue) -> Result<v1::Value> {
    let kind = match value {
        ScalarValue::Missing | ScalarValue::Null => v1::value::Kind::Null(v1::Null {}),
        ScalarValue::Float(v) => v1::value::Kind::Number(*v),
        ScalarValue::Signed(v) => v1::value::Kind::Integer(*v),
        ScalarValue::Unsigned(v) => v1::value::Kind::UnsignedInteger(*v),
        ScalarValue::Bool(v) => v1::value::Kind::Boolean(*v),
        ScalarValue::Text(v) => v1::value::Kind::Text(v.clone()),
        ScalarValue::Exact(v) if v.denominator.get() == 1 && v.negative && v.numerator <= i64::MAX as u128 + 1 => {
            v1::value::Kind::Integer(-(v.numerator as i128) as i64)
        }
        ScalarValue::Exact(v) if v.denominator.get() == 1 && !v.negative && v.numerator <= u64::MAX as u128 => {
            v1::value::Kind::UnsignedInteger(v.numerator as u64)
        }
        ScalarValue::Exact(_) => v1::value::Kind::Number(number(value).unwrap()),
        _ => return Err(Error::Data("select a scalar value expressible by the product protocol")),
    };
    Ok(v1::Value { kind: Some(kind) })
}
pub(crate) fn text(value: &ScalarValue) -> String {
    match value {
        ScalarValue::Text(v) => v.clone(),
        ScalarValue::Signed(v) => v.to_string(),
        ScalarValue::Unsigned(v) => v.to_string(),
        ScalarValue::Float(v) => v.to_string(),
        ScalarValue::Bool(v) => v.to_string(),
        _ => String::new(),
    }
}

/* === Private Functions === */

/* === Tests === */

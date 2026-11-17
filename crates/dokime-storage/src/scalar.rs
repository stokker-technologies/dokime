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

pub(crate) use harmos_query::scalar::{Agg, ScalarValue};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result};
use dokime_protocol::model;
use dokime_protocol::model::Null;
use dokime_protocol::model::Value;
use harmos_query::scalar::ExactNumber;
use std::num::NonZeroU128;

use dokime_protocol::model::value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

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
pub(crate) fn from_wire(value: &Value) -> Result<ScalarValue> {
    Ok(match value.kind.as_ref() {
        Some(value::Kind::Null(_)) => ScalarValue::Null,
        Some(value::Kind::Number(v)) if v.is_finite() => ScalarValue::Float(*v),
        Some(value::Kind::Integer(v)) => ScalarValue::Signed(*v),
        Some(value::Kind::UnsignedInteger(v)) => ScalarValue::Unsigned(*v),
        Some(value::Kind::Exact(v)) => ScalarValue::Exact(ExactNumber::new(
            v.negative,
            v.numerator.parse().map_err(|_| Error::Data("Read Exact Value: Supply an Unsigned Numerator"))?,
            v.denominator
                .parse::<u128>()
                .ok()
                .and_then(NonZeroU128::new)
                .ok_or(Error::Data("Read Exact Value: Supply a Positive Denominator"))?,
        )),
        Some(value::Kind::Boolean(v)) => ScalarValue::Bool(*v),
        Some(value::Kind::Text(v)) => ScalarValue::Text(v.clone()),
        _ => return Err(Error::Data("select a finite scalar value")),
    })
}
pub(crate) fn wire(value: &ScalarValue) -> Result<Value> {
    let kind = match value {
        ScalarValue::Missing | ScalarValue::Null => value::Kind::Null(Null {}),
        ScalarValue::Float(v) if v.is_finite() => value::Kind::Number(*v),
        ScalarValue::Signed(v) => value::Kind::Integer(*v),
        ScalarValue::Unsigned(v) => value::Kind::UnsignedInteger(*v),
        ScalarValue::Bool(v) => value::Kind::Boolean(*v),
        ScalarValue::Text(v) => value::Kind::Text(v.clone()),
        ScalarValue::Exact(v) if v.denominator.get() == 1 && v.negative && v.numerator <= i64::MAX as u128 + 1 => {
            value::Kind::Integer(-(v.numerator as i128) as i64)
        }
        ScalarValue::Exact(v) if v.denominator.get() == 1 && !v.negative && v.numerator <= u64::MAX as u128 => {
            value::Kind::UnsignedInteger(v.numerator as u64)
        }
        ScalarValue::Exact(v) => value::Kind::Exact(model::ExactNumber {
            negative: v.negative,
            numerator: v.numerator.to_string(),
            denominator: v.denominator.to_string(),
        }),
        _ => return Err(Error::Data("select a scalar value expressible by the product protocol")),
    };
    Ok(Value { kind: Some(kind) })
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

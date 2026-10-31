/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Captured Query Predicate

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::cmp::Ordering;

// Third Party Imports

// Local Imports
use super::FieldRef;
use dokime_domain::model::{
    query::{
        error::{Failure, Reason},
        predicate::{Operator, Predicate, Scalar},
        schema::FieldType,
    },
    values::{Value, numeric_cmp},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct PreparedPredicate {
    pub(super) field: FieldRef,
    operator: Operator,
    values: Vec<Value>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PreparedPredicate {
    pub(super) fn new(predicate: &Predicate, field: FieldRef, kind: FieldType, operation: &str) -> Result<Self, Box<Failure>> {
        predicate.validate_shape(operation)?;
        let permitted = match kind {
            FieldType::Boolean | FieldType::Severity => matches!(predicate.operator, Operator::Eq | Operator::In | Operator::Exists),
            FieldType::Object | FieldType::Array => predicate.operator == Operator::Exists,
            _ => true,
        };
        if !permitted {
            return Err(mismatch(predicate, operation));
        }
        if predicate.operator != Operator::Exists && !predicate.values.iter().all(|value| compatible(kind, value)) {
            return Err(mismatch(predicate, operation));
        }
        let values = predicate.values.iter().map(core_value).collect();
        Ok(Self {
            field,
            operator: predicate.operator,
            values,
        })
    }

    pub(super) fn matches(&self, value: Option<&Value>) -> bool {
        let present = value.is_some_and(|value| !matches!(value, Value::Null));
        if self.operator == Operator::Exists {
            return present == matches!(self.values.as_slice(), [Value::Bool(true)]);
        }
        let Some(value) = value.filter(|_| present) else {
            return false;
        };
        self.values.iter().any(|operand| {
            let ordering = compare(value, operand);
            match self.operator {
                Operator::Eq | Operator::In => ordering == Some(Ordering::Equal),
                Operator::Lt => ordering == Some(Ordering::Less),
                Operator::Lte => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
                Operator::Gt => ordering == Some(Ordering::Greater),
                Operator::Gte => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
                Operator::Exists => unreachable!(),
            }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn compatible(kind: FieldType, value: &Scalar) -> bool {
    match kind {
        FieldType::Integer | FieldType::Int64 | FieldType::Uint64 | FieldType::Float64 => {
            matches!(value, Scalar::Integer(_) | Scalar::Unsigned(_) | Scalar::Number(_))
        }
        FieldType::Boolean => matches!(value, Scalar::Boolean(_)),
        FieldType::Text => matches!(value, Scalar::Text(_)),
        FieldType::Datetime => matches!(value, Scalar::Datetime(_)),
        FieldType::Severity => matches!(value, Scalar::Severity(_)),
        FieldType::Object | FieldType::Array => false,
    }
}

fn core_value(value: &Scalar) -> Value {
    match value {
        Scalar::Boolean(value) => Value::Bool(*value),
        Scalar::Integer(value) => Value::Integer(*value),
        Scalar::Unsigned(value) => Value::Unsigned(*value),
        Scalar::Number(value) => Value::Number(*value),
        Scalar::Text(value) => Value::Text(value.clone()),
        Scalar::Datetime(value) => Value::Datetime(value.to_datetime().expect("validated operand timestamp")),
        Scalar::Severity(value) => Value::Text(format!("{value:?}")),
    }
}

fn compare(value: &Value, operand: &Value) -> Option<Ordering> {
    match (value, operand) {
        (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
        (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
        (Value::Datetime(a), Value::Datetime(b)) => Some(a.cmp(b)),
        _ => numeric_cmp(value, operand),
    }
}

fn mismatch(predicate: &Predicate, operation: &str) -> Box<Failure> {
    let mut error = Failure::new(
        Reason::TypeMismatch,
        operation,
        "Operand or operator is incompatible with the captured field type; use exists for null or structured values",
    );
    error.field = Some(predicate.field.clone());
    error.operator = Some(predicate.operator);
    error
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_integers() {
        for (operator, operand, value, expected) in [
            (Operator::Eq, Scalar::Unsigned(u64::MAX), Value::Unsigned(u64::MAX), true),
            (Operator::Eq, Scalar::Unsigned(u64::MAX), Value::Unsigned(u64::MAX - 1), false),
            (Operator::Eq, Scalar::Unsigned(9_007_199_254_740_993), Value::Unsigned(9_007_199_254_740_992), false),
            (Operator::Gt, Scalar::Number(9_007_199_254_740_992.0), Value::Unsigned(9_007_199_254_740_993), true),
            (Operator::Eq, Scalar::Number(9_007_199_254_740_992.0), Value::Unsigned(9_007_199_254_740_993), false),
            (Operator::Lt, Scalar::Unsigned(0), Value::Integer(-1), true),
            (Operator::Lt, Scalar::Number(-0.5), Value::Integer(-1), true),
            (Operator::Lt, Scalar::Number(18_446_744_073_709_551_616.0), Value::Unsigned(u64::MAX), true),
            (Operator::Gt, Scalar::Integer(i64::MAX), Value::Unsigned(u64::MAX), true),
            (Operator::Eq, Scalar::Number(0.0), Value::Bool(false), false),
        ] {
            let predicate = Predicate {
                field: "counter".into(),
                operator,
                values: vec![operand],
            };
            let prepared =
                PreparedPredicate::new(&predicate, FieldRef::Declared("counter".into()), FieldType::Integer, "test.query").unwrap();
            assert_eq!(prepared.matches(Some(&value)), expected);
        }
    }
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Predicate Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::error::{Failure, Reason};
use crate::model::identity::wire::{Severity, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub field: String,
    pub operator: Operator,
    pub values: Vec<Scalar>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
    In,
    Exists,
}

// Null Has No Operand Representation; Use Exists(false).
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Scalar {
    Boolean(bool),
    Integer(i64),
    Unsigned(u64),
    Number(f64),
    Text(String),
    Datetime(Timestamp),
    Severity(Severity),
}

/* === Traits === */

/* === Implementations === */

impl Predicate {
    pub fn validate_shape(&self, operation: &str) -> Result<(), Box<Failure>> {
        let valid = !self.field.is_empty()
            && match self.operator {
                Operator::In => !self.values.is_empty(),
                Operator::Exists => matches!(self.values.as_slice(), [Scalar::Boolean(_)]),
                _ => self.values.len() == 1,
            };
        let finite = self.values.iter().all(|v| match v {
            Scalar::Number(v) => v.is_finite(),
            Scalar::Datetime(v) => v.to_datetime().is_ok(),
            Scalar::Severity(v) => *v != Severity::Unspecified,
            _ => true,
        });
        if valid && finite {
            return Ok(());
        }
        let mut failure = Failure::new(
            Reason::InvalidOperand,
            operation,
            "Use one typed scalar, nonempty IN values, or one boolean for exists; null uses exists(false)",
        );
        failure.field = Some(self.field.clone());
        failure.operator = Some(self.operator);
        Err(failure)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operand_shapes() {
        for (operator, values, valid) in [
            (Operator::Eq, vec![], false),
            (Operator::In, vec![], false),
            (Operator::Exists, vec![Scalar::Integer(1)], false),
            (Operator::Eq, vec![Scalar::Number(f64::INFINITY)], false),
            (Operator::Exists, vec![Scalar::Boolean(false)], true),
            (Operator::In, vec![Scalar::Unsigned(u64::MAX)], true),
        ] {
            let predicate = Predicate {
                field: "counter".into(),
                operator,
                values,
            };
            assert_eq!(predicate.validate_shape("test.query").is_ok(), valid);
        }
        assert!(serde_json::from_str::<Operator>(r#""sql""#).is_err());
        assert!(serde_json::from_str::<Scalar>("null").is_err());
    }
}

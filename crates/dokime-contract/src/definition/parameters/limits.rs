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
use crate::core::Value;
use crate::definition::{ParameterDefinition, ParameterExpr};

/* === Constants === */
pub const BUDGET: usize = 8 * 1024 * 1024;

/* === Macros === */
product_error!(ParameterLimitError, formatter {
 Self::Structure => write!(formatter,"parameter value/expression structural budget exceeded"),
 Self::Expansion => write!(formatter,"parameter expansion exceeds limits or cannot advance safely"),
 Self::Payload => write!(formatter,"parameter materialization exceeds allocation budget"),
 Self::Invalid(message) => write!(formatter,"{message}"),
});

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub struct Shape {
    pub count: usize,
    pub cost: usize,
    pub space: bool,
}

/* === Enums === */

#[derive(Debug, PartialEq)]
pub enum ParameterLimitError {
    Structure,
    Expansion,
    Payload,
    Invalid(&'static str),
}

/* === Traits === */

/* === Implementations === */
impl Shape {
    fn scalar(cost: usize) -> Self {
        Self {
            count: 1,
            cost,
            space: false,
        }
    }
}

/* === Public Functions === */
pub fn charge(remaining: &mut usize, amount: usize) -> crate::Result<()> {
    *remaining = remaining.checked_sub(amount).ok_or_else(limit)?;
    Ok(())
}

pub fn parameter_shape(
    definition: &ParameterDefinition,
    definitions: &[ParameterDefinition],
    overrides: &BTreeMap<String, Vec<Value>>,
    depth: usize,
    remaining: &mut usize,
) -> crate::Result<Shape> {
    let shape = if let Some(values) = overrides.get(definition.metadata.identifier.as_str()) {
        Shape {
            count: values.len(),
            cost: values.iter().map(value_cost).max().unwrap_or(256),
            space: true,
        }
    } else {
        expression_shape(
            &definition.expression,
            definitions,
            overrides,
            depth,
            remaining,
        )?
    };
    charge(remaining, shape.count.saturating_mul(shape.cost))?;
    Ok(shape)
}

/* === Private Functions === */
fn limit() -> crate::Error {
    ParameterLimitError::Payload.into()
}

fn value_cost(value: &Value) -> usize {
    // Allow for Value/map/vector allocation and copies into generated labels;
    // text receives space for escaped rendering and ownership copies.
    256_usize.saturating_add(match value {
        Value::Text(value) => value.len().saturating_mul(8),
        Value::Array(values) => values.iter().fold(0_usize, |cost, value| {
            cost.saturating_add(value_cost(value))
        }),
        Value::Object(values) => values.iter().fold(0_usize, |cost, (key, value)| {
            cost.saturating_add(key.len().saturating_mul(8))
                .saturating_add(value_cost(value))
        }),
        _ => 0,
    })
}

fn expression_shape(
    expr: &ParameterExpr,
    definitions: &[ParameterDefinition],
    overrides: &BTreeMap<String, Vec<Value>>,
    depth: usize,
    remaining: &mut usize,
) -> crate::Result<Shape> {
    if depth > 32 {
        return Err(limit());
    }
    let shape = match expr {
        ParameterExpr::Literal { value } => Shape::scalar(value_cost(value)),
        ParameterExpr::Array { values } => {
            let mut cost = 256;
            for value in values {
                let item = expression_shape(value, definitions, overrides, depth + 1, remaining)?;
                if item.space {
                    return Err(ParameterLimitError::Invalid(
                        "array expression items must be scalar",
                    )
                    .into());
                }
                cost = cost.max(item.cost);
            }
            Shape {
                count: values.len(),
                cost,
                space: true,
            }
        }
        ParameterExpr::Range { start, stop, step } => {
            // Numeric validity and expansion count were checked by bounds.rs.
            let distance = if *step > 0 {
                i128::from(*stop) - i128::from(*start)
            } else {
                i128::from(*start) - i128::from(*stop)
            };
            let count = if distance < 0 {
                0
            } else {
                (distance / i128::from(*step).abs() + 1) as usize
            };
            Shape {
                count,
                cost: 256,
                space: true,
            }
        }
        ParameterExpr::Arange { start, stop, step } => {
            let mut current = *start;
            let mut count = 0;
            while if *step > 0.0 {
                current < *stop
            } else {
                current > *stop
            } {
                count += 1;
                current += step;
            }
            Shape {
                count,
                cost: 256,
                space: true,
            }
        }
        ParameterExpr::Linspace { count, .. } => Shape {
            count: *count,
            cost: 256,
            space: true,
        },
        ParameterExpr::Reference { parameter } => {
            let definition = definitions
                .iter()
                .find(|definition| definition.metadata.identifier.as_str() == parameter)
                .ok_or_else(|| {
                    crate::Error::from(ParameterLimitError::Invalid("unknown parameter reference"))
                })?;
            parameter_shape(definition, definitions, overrides, depth + 1, remaining)?
        }
        ParameterExpr::DynamicDefault { fallback, .. } => match fallback {
            Some(fallback) => {
                expression_shape(fallback, definitions, overrides, depth + 1, remaining)?
            }
            None => Shape::scalar(256),
        },
        ParameterExpr::Unary { value, .. } => {
            expression_shape(value, definitions, overrides, depth + 1, remaining)?
        }
        ParameterExpr::Binary { left, right, .. } => {
            let left = expression_shape(left, definitions, overrides, depth + 1, remaining)?;
            let right = expression_shape(right, definitions, overrides, depth + 1, remaining)?;
            Shape {
                count: left.count.max(right.count),
                cost: left.cost.saturating_add(right.cost),
                space: left.space || right.space,
            }
        }
    };
    charge(remaining, shape.count.saturating_mul(shape.cost))?;
    Ok(shape)
}

/* === Tests === */

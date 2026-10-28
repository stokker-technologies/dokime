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

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::PromptDefinition;
use crate::model::diagnostics::ValidationIssue;
use crate::model::expressions::BooleanExpression;
use crate::model::expressions::Expression;
use crate::model::expressions::ExpressionBinaryOp;
use crate::model::expressions::ExpressionKind;
use crate::model::values::Value;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub predicate: BooleanExpression,
    pub message: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for Rule {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.predicate), std::mem::size_of_val(&self.predicate)),
            (harmos_domain::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */

pub fn validate_rules(definition: &PromptDefinition) -> Vec<ValidationIssue> {
    if definition.rules.len() > 32 {
        return vec![issue(0, "a prompt may declare at most 32 answer rules")];
    }
    definition
        .rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            let error = if rule.message.trim().is_empty() {
                Some("a rule requires a failure message")
            } else {
                match expression_type(&rule.predicate.expression, definition, 1, &mut 128) {
                    Ok(ValueType::Boolean) => None,
                    Ok(_) => Some("a rule must evaluate to boolean"),
                    Err(error) => Some(error),
                }
            };
            error.map(|message| issue(index, message))
        })
        .collect()
}

/* === Private Functions === */

fn expression_type(
    expression: &Expression, definition: &PromptDefinition, depth: usize, remaining: &mut usize,
) -> Result<ValueType, &'static str> {
    if depth > 16 || *remaining == 0 {
        return Err("a rule exceeds 16 levels or 128 expression nodes");
    }
    *remaining -= 1;
    match &expression.kind {
        ExpressionKind::Literal { value } => match value {
            Value::Bool(_) => Ok(ValueType::Boolean),
            Value::Integer(_) | Value::Unsigned(_) => Ok(ValueType::Integer),
            Value::Number(value) if value.is_finite() => Ok(ValueType::Number),
            Value::Text(_) => Ok(ValueType::Text),
            _ => Err("rule literals must be finite numbers, text, or booleans"),
        },
        ExpressionKind::Selector { path } => {
            let field = definition
                .fields
                .iter()
                .find(|field| field.metadata.identifier.as_str() == path)
                .ok_or("rule selector must name a declared answer field")?;
            if !field.required {
                return Err("rule selectors must name required answer fields");
            }
            Ok(if field.multiple { ValueType::Array } else { field.value_type.clone() })
        }
        ExpressionKind::Call { .. } => Err("function calls are not supported in answer rules"),
        ExpressionKind::Unary { value, .. } => {
            if expression_type(value, definition, depth + 1, remaining)? != ValueType::Boolean {
                return Err("not requires a boolean operand");
            }
            Ok(ValueType::Boolean)
        }
        ExpressionKind::Binary { op, left, right } => {
            let left = expression_type(left, definition, depth + 1, remaining)?;
            let right = expression_type(right, definition, depth + 1, remaining)?;
            use ExpressionBinaryOp::*;
            let valid = match op {
                Eq | Ne => true, // Preserve structural equality, including unlike types.
                And | Or => left == ValueType::Boolean && right == ValueType::Boolean,
                Gt | Gte | Lt | Lte => {
                    matches!(left, ValueType::Integer | ValueType::Number) && matches!(right, ValueType::Integer | ValueType::Number)
                }
                Contains | StartsWith | EndsWith => left == ValueType::Text && right == ValueType::Text,
            };
            if valid { Ok(ValueType::Boolean) } else { Err("rule operator has incompatible operand types") }
        }
    }
}

fn issue(index: usize, message: &str) -> ValidationIssue {
    ValidationIssue {
        path: format!("rules.{index}"),
        message: message.into(),
        expected: None,
    }
}

/* === Tests === */

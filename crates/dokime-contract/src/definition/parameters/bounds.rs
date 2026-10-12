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

// Local Imports
use super::limits::ParameterLimitError;
use crate::core::Value;
use crate::definition::ParameterExpr;

/* === Constants === */
const MAX_SPACE: usize = 1000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Budget {
    nodes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Budget {
    fn default() -> Self {
        Self::new()
    }
}

impl Budget {
    pub fn new() -> Self {
        Self { nodes: 4096 }
    }
    pub fn node(&mut self, depth: usize) -> crate::Result<()> {
        if depth > 32 || self.nodes == 0 {
            return Err(ParameterLimitError::Structure.into());
        }
        self.nodes -= 1;
        Ok(())
    }
    pub fn text(value: &str) -> crate::Result<()> {
        if value.len() > 4096 {
            return Err(ParameterLimitError::Structure.into());
        }
        Ok(())
    }
    pub fn value(&mut self, value: &Value, depth: usize) -> crate::Result<()> {
        self.node(depth)?;
        match value {
            Value::Number(number) if !number.is_finite() => {
                return Err(ParameterLimitError::Invalid("number must be finite").into());
            }
            Value::Text(text) => Self::text(text)?,
            Value::Array(values) => {
                if values.len() > 256 {
                    return Err(ParameterLimitError::Structure.into());
                }
                for value in values {
                    self.value(value, depth + 1)?;
                }
            }
            Value::Object(values) => {
                if values.len() > 256 {
                    return Err(ParameterLimitError::Structure.into());
                }
                for (key, value) in values {
                    Self::text(key)?;
                    self.value(value, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/* === Public Functions === */
pub fn expression(expr: &ParameterExpr, budget: &mut Budget, depth: usize) -> crate::Result<()> {
    walk(expr, budget, depth, true)
}

pub fn structure(expr: &ParameterExpr, budget: &mut Budget, depth: usize) -> crate::Result<()> {
    walk(expr, budget, depth, false)
}

pub fn definition(
    definition: &crate::definition::ParameterDefinition,
    budget: &mut Budget,
) -> crate::Result<()> {
    budget.node(0)?;
    metadata(&definition.metadata)?;
    super::validate_numeric_bounds(definition, &Value::Null).map_err(|_| {
        crate::Error::from(ParameterLimitError::Invalid(
            "parameter numeric bounds must be finite, ordered, and attached to a numeric type",
        ))
    })?;
    if let super::ParameterExpansion::Zip { group } = &definition.expansion {
        Budget::text(group.as_str())?;
    }
    if let Some(group) = &definition.group_id {
        Budget::text(group.as_str())?;
    }
    if let Some(unit) = &definition.unit {
        metadata(&unit.metadata)?;
        Budget::text(&unit.symbol)?;
    }
    structure(&definition.expression, budget, 0)?;
    if let Some(selection) = &definition.selection {
        if selection.choices.len() > 256 {
            return Err(ParameterLimitError::Structure.into());
        }
        for value in &selection.choices {
            budget.value(value, 0)?;
        }
    }
    if definition.extensions.len() > 256 {
        return Err(ParameterLimitError::Structure.into());
    }
    for (key, value) in &definition.extensions {
        Budget::text(key)?;
        budget.value(value, 0)?;
    }
    Ok(())
}

/* === Private Functions === */
fn walk(
    expr: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
    expansion: bool,
) -> crate::Result<()> {
    budget.node(depth)?;
    match expr {
        ParameterExpr::Literal { value: literal } => {
            budget.value(literal, depth + 1)?;
        }
        ParameterExpr::Array { values } => {
            if values.len() > if expansion { MAX_SPACE } else { 256 } {
                return Err(limit());
            }
            for item in values {
                walk(item, budget, depth + 1, expansion)?;
            }
        }
        ParameterExpr::Range { start, stop, step } => {
            if !expansion {
                return Ok(());
            }
            if *step == 0 {
                return Err(limit());
            }
            let (mut current, stop, step) =
                (i128::from(*start), i128::from(*stop), i128::from(*step));
            let mut count = 0;
            while if step > 0 {
                current <= stop
            } else {
                current >= stop
            } {
                count += 1;
                if count > MAX_SPACE {
                    return Err(limit());
                }
                current += step;
                // The native descending loop adds its step after the last
                // element too; do not let that existing path overflow i64.
                if step < 0 && current < i128::from(i64::MIN) {
                    return Err(limit());
                }
            }
        }
        ParameterExpr::Arange { start, stop, step } => {
            if !expansion {
                return if start.is_finite() && stop.is_finite() && step.is_finite() {
                    Ok(())
                } else {
                    Err(limit())
                };
            }
            if !start.is_finite() || !stop.is_finite() || !step.is_finite() || *step == 0.0 {
                return Err(limit());
            }
            let mut current = *start;
            let mut count = 0;
            while if *step > 0.0 {
                current < *stop
            } else {
                current > *stop
            } {
                count += 1;
                if count > MAX_SPACE {
                    return Err(limit());
                }
                let next = current + step;
                if !next.is_finite() || next == current {
                    return Err(limit());
                }
                current = next;
            }
        }
        ParameterExpr::Linspace { start, stop, count } => {
            if (expansion && *count > MAX_SPACE)
                || !start.is_finite()
                || !stop.is_finite()
                || (expansion && !(stop - start).is_finite())
            {
                return Err(limit());
            }
        }
        ParameterExpr::Reference { parameter } => Budget::text(parameter)?,
        ParameterExpr::DynamicDefault { fallback, .. } => {
            if let Some(fallback) = fallback {
                walk(fallback, budget, depth + 1, expansion)?;
            }
        }
        ParameterExpr::Unary { value, .. } => walk(value, budget, depth + 1, expansion)?,
        ParameterExpr::Binary { left, right, .. } => {
            walk(left, budget, depth + 1, expansion)?;
            walk(right, budget, depth + 1, expansion)?;
        }
    }
    Ok(())
}

fn limit() -> crate::Error {
    ParameterLimitError::Expansion.into()
}

fn metadata(metadata: &crate::core::Metadata) -> crate::Result<()> {
    Budget::text(metadata.identifier.as_str())?;
    Budget::text(&metadata.label)?;
    if let Some(description) = &metadata.description {
        Budget::text(description)?;
    }
    let mut budget = Budget::new();
    for tag in &metadata.tags {
        Budget::text(tag)?;
        budget.node(0)?;
    }
    for (key, value) in &metadata.attributes {
        Budget::text(key)?;
        budget.value(value, 0)?;
    }
    Ok(())
}

/* === Tests === */

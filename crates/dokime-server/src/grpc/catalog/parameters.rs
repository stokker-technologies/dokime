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
use tonic::Status;

// Local Imports
use super::super::{
    enums,
    value::{self, Budget, projection},
};
use dokime_app::core::{MeasurementUnit, UnitFamily};
use dokime_app::definition::{
    DynamicDefaultKey, ParameterBinaryOp, ParameterDefinition, ParameterExpansion, ParameterExpr,
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::grpc) fn memory(
    parameters: &[ParameterDefinition],
    budget: &mut Budget,
) -> harmos::Result<usize> {
    value::collection(parameters.len()).map_err(|_| exhausted())?;
    let mut bytes = std::mem::size_of::<Vec<v1::ParameterSchema>>().saturating_add(
        parameters
            .len()
            .saturating_mul(std::mem::size_of::<v1::ParameterSchema>()),
    );
    for parameter in parameters {
        budget.node(0).map_err(|_| exhausted())?;
        bytes = bytes
            .saturating_add(parameter.parameter_id.as_str().len())
            .saturating_add(
                projection::metadata_fields_count(&parameter.metadata)?.memory
                    - std::mem::size_of::<v1::Metadata>(),
            )
            .saturating_add(
                parameter
                    .group_id
                    .as_ref()
                    .map_or(0, |id| id.as_str().len()),
            )
            .saturating_add(expression_memory(&parameter.expression, budget, 0)?);
        if let ParameterExpansion::Zip { group } = &parameter.expansion {
            bytes = bytes.saturating_add(group.as_str().len());
        }
        if let Some(source) = &parameter.unit {
            bytes = bytes
                .saturating_add(unit_memory(source)? - std::mem::size_of::<v1::MeasurementUnit>());
        }
        value::collection(parameter.extensions.len()).map_err(|_| exhausted())?;
        let nodes = harmos::btree_map_memory::<String, v1::Value>(parameter.extensions.len());
        if nodes == usize::MAX {
            return Err(exhausted());
        }
        bytes = bytes.saturating_add(
            nodes - std::mem::size_of::<std::collections::BTreeMap<String, v1::Value>>(),
        );
        for (key, item) in &parameter.extensions {
            value::text(key).map_err(|_| exhausted())?;
            bytes = bytes.saturating_add(key.len()).saturating_add(
                projection::count(item, budget, 0)?.memory - std::mem::size_of::<v1::Value>(),
            );
        }
        if let Some(selection) = &parameter.selection {
            value::collection(selection.choices.len()).map_err(|_| exhausted())?;
            for choice in &selection.choices {
                bytes = bytes.saturating_add(projection::count(choice, budget, 0)?.memory);
            }
        }
    }
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes)
}

pub(in crate::grpc) fn unit_memory(source: &MeasurementUnit) -> harmos::Result<usize> {
    value::text(&source.symbol).map_err(|_| exhausted())?;
    let metadata =
        projection::metadata_count(&source.metadata)?.memory - std::mem::size_of::<v1::Metadata>();
    std::mem::size_of::<v1::MeasurementUnit>()
        .checked_add(source.symbol.len())
        .and_then(|bytes| bytes.checked_add(metadata))
        .ok_or_else(exhausted)
}

pub(in crate::grpc) fn wire(
    parameters: &[ParameterDefinition],
    budget: &mut Budget,
) -> Result<Vec<v1::ParameterSchema>, Status> {
    value::collection(parameters.len())?;
    collect(parameters.iter().map(|parameter| {
        budget.node(0)?;
        value::collection(parameter.extensions.len())?;
        let mut extensions = std::collections::BTreeMap::new();
        for (key, item) in &parameter.extensions {
            value::text(key)?;
            extensions.insert(
                projection::text(key).map_err(failure)?,
                project_value(item, budget, 0)?,
            );
        }
        let selection = parameter
            .selection
            .as_ref()
            .map(|selection| {
                value::collection(selection.choices.len())?;
                Ok::<_, Status>(v1::ParameterSelection {
                    choices: collect(
                        selection
                            .choices
                            .iter()
                            .map(|value| project_value(value, budget, 0)),
                    )?,
                })
            })
            .transpose()?;
        let kind = match &parameter.expansion {
            dokime_app::definition::ParameterExpansion::Single => {
                v1::parameter_expansion::Kind::Single(true)
            }
            dokime_app::definition::ParameterExpansion::Cartesian => {
                v1::parameter_expansion::Kind::Cartesian(true)
            }
            dokime_app::definition::ParameterExpansion::Zip { group } => {
                v1::parameter_expansion::Kind::Zip(group.as_str().to_owned())
            }
        };
        Ok(v1::ParameterSchema {
            expansion: Some(v1::ParameterExpansion { kind: Some(kind) }),
            parameter_id: parameter.parameter_id.as_str().to_owned(),
            metadata: Some(metadata(&parameter.metadata, false)?),
            group_id: parameter.group_id.as_ref().map(|id| id.as_str().to_owned()),
            value_type: enums::value_type(&parameter.value_type) as i32,
            unit: parameter.unit.as_ref().map(unit).transpose()?,
            required: parameter.required,
            runtime_writable: parameter.runtime_writable,
            min: parameter.min,
            max: parameter.max,
            expression: Some(expression(&parameter.expression, budget, 0)?),
            selection,
            list: parameter.list.as_ref().map(|list| v1::ParameterList {
                element_type: enums::value_type(&list.element_type) as i32,
                min_items: list.min_items.map(|items| items as u32),
                max_items: list.max_items.map(|items| items as u32),
            }),
            extensions,
        })
    }))
}

pub(in crate::grpc) fn unit(unit: &MeasurementUnit) -> Result<v1::MeasurementUnit, Status> {
    value::text(&unit.symbol)?;
    Ok(v1::MeasurementUnit {
        metadata: Some(metadata(&unit.metadata, true)?),
        symbol: unit.symbol.clone(),
        family: match unit.family {
            UnitFamily::Count => v1::UnitFamily::Count,
            UnitFamily::Distance => v1::UnitFamily::Distance,
            UnitFamily::Duration => v1::UnitFamily::Duration,
            UnitFamily::Frequency => v1::UnitFamily::Frequency,
            UnitFamily::Power => v1::UnitFamily::Power,
            UnitFamily::Temperature => v1::UnitFamily::Temperature,
            UnitFamily::Voltage => v1::UnitFamily::Voltage,
            UnitFamily::Ratio => v1::UnitFamily::Ratio,
            UnitFamily::Custom => v1::UnitFamily::Custom,
        } as i32,
    })
}

fn finite(values: &[f64]) -> Result<(), Status> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(Status::internal("catalog contains nonfinite expression"))
    } else {
        Ok(())
    }
}

fn expression(
    expr: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::ParameterExpression, Status> {
    use v1::parameter_expression::Kind;
    budget.node(depth)?;
    Ok(v1::ParameterExpression {
        kind: Some(match expr {
            ParameterExpr::Literal { value } => {
                Kind::Literal(project_value(value, budget, depth + 1)?)
            }
            ParameterExpr::Array { values } => {
                value::collection(values.len())?;
                Kind::Array(v1::ParameterExpressions {
                    values: collect(
                        values
                            .iter()
                            .map(|expr| expression(expr, budget, depth + 1)),
                    )?,
                })
            }
            ParameterExpr::Range { start, stop, step } => Kind::Range(v1::IntegerRange {
                start: *start,
                stop: *stop,
                step: *step,
            }),
            ParameterExpr::Arange { start, stop, step } => {
                finite(&[*start, *stop, *step])?;
                Kind::Arange(v1::NumberRange {
                    start: *start,
                    stop: *stop,
                    step: *step,
                })
            }
            ParameterExpr::Linspace { start, stop, count } => {
                finite(&[*start, *stop])?;
                Kind::Linspace(v1::LinearSpace {
                    start: *start,
                    stop: *stop,
                    count: *count as u64,
                })
            }
            ParameterExpr::Reference { parameter } => {
                value::text(parameter)?;
                Kind::Reference(parameter.clone())
            }
            ParameterExpr::DynamicDefault { key, fallback } => {
                Kind::DynamicDefault(Box::new(v1::DynamicDefault {
                    key: match key {
                        DynamicDefaultKey::OperatorName => v1::DynamicDefaultKey::OperatorName,
                        DynamicDefaultKey::OperatorEmail => v1::DynamicDefaultKey::OperatorEmail,
                    } as i32,
                    fallback: fallback
                        .as_ref()
                        .map(|expr| expression(expr, budget, depth + 1).map(Box::new))
                        .transpose()?,
                }))
            }
            ParameterExpr::Unary { value, .. } => Kind::Unary(Box::new(v1::UnaryExpression {
                op: v1::UnaryOperator::Negate as i32,
                value: Some(Box::new(expression(value, budget, depth + 1)?)),
            })),
            ParameterExpr::Binary { op, left, right } => {
                Kind::Binary(Box::new(v1::BinaryExpression {
                    op: match op {
                        ParameterBinaryOp::Add => v1::BinaryOperator::Add,
                        ParameterBinaryOp::Subtract => v1::BinaryOperator::Subtract,
                        ParameterBinaryOp::Multiply => v1::BinaryOperator::Multiply,
                        ParameterBinaryOp::Divide => v1::BinaryOperator::Divide,
                    } as i32,
                    left: Some(Box::new(expression(left, budget, depth + 1)?)),
                    right: Some(Box::new(expression(right, budget, depth + 1)?)),
                }))
            }
        }),
    })
}

/* === Private Functions === */

fn expression_memory(
    source: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<usize> {
    budget.node(depth).map_err(|_| exhausted())?;
    let bytes = match source {
        ParameterExpr::Literal { value } => {
            projection::count(value, budget, depth + 1)?.memory - std::mem::size_of::<v1::Value>()
        }
        ParameterExpr::Array { values } => {
            value::collection(values.len()).map_err(|_| exhausted())?;
            let mut bytes = values
                .len()
                .saturating_mul(std::mem::size_of::<v1::ParameterExpression>());
            for value in values {
                bytes = bytes.saturating_add(expression_memory(value, budget, depth + 1)?);
            }
            bytes
        }
        ParameterExpr::Reference { parameter } => {
            value::text(parameter).map_err(|_| exhausted())?;
            parameter.len()
        }
        ParameterExpr::DynamicDefault { fallback, .. } => {
            let mut bytes = std::mem::size_of::<v1::DynamicDefault>();
            if let Some(fallback) = fallback {
                bytes = bytes
                    .saturating_add(std::mem::size_of::<v1::ParameterExpression>())
                    .saturating_add(expression_memory(fallback, budget, depth + 1)?);
            }
            bytes
        }
        ParameterExpr::Unary { value, .. } => std::mem::size_of::<v1::UnaryExpression>()
            .saturating_add(std::mem::size_of::<v1::ParameterExpression>())
            .saturating_add(expression_memory(value, budget, depth + 1)?),
        ParameterExpr::Binary { left, right, .. } => std::mem::size_of::<v1::BinaryExpression>()
            .saturating_add(std::mem::size_of::<v1::ParameterExpression>())
            .saturating_add(expression_memory(left, budget, depth + 1)?)
            .saturating_add(std::mem::size_of::<v1::ParameterExpression>())
            .saturating_add(expression_memory(right, budget, depth + 1)?),
        ParameterExpr::Range { .. }
        | ParameterExpr::Arange { .. }
        | ParameterExpr::Linspace { .. } => 0,
    };
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes)
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical parameter construction",
    }
}

fn failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("canonical parameter construction exceeds its bound")
        }
        _ => Status::internal("canonical parameter projection failed"),
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = projection::slots(source.len()).map_err(failure)?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata(source: &dokime_app::core::Metadata, checked: bool) -> Result<v1::Metadata, Status> {
    if checked {
        projection::metadata_count(source).map_err(failure)?;
    } else {
        projection::metadata_fields_count(source).map_err(failure)?;
    }
    projection::metadata(source).map_err(failure)
}

fn project_value(
    source: &dokime_app::core::Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::Value, Status> {
    projection::count(source, &mut Budget::new(), depth).map_err(failure)?;
    projection::project(source, budget, depth).map_err(failure)
}

/* === Tests === */

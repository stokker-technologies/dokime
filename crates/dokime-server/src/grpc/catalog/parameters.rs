/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog parameter projection onto the wire, with its memory accounting and value checks.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::value::canonical;
use super::super::value::{self, Budget};
use dokime_domain::model::values::MeasurementUnit;
use dokime_domain::model::values::UnitFamily;
use dokime_runtime::definition::{
    DynamicDefaultKey, ParameterBinaryOp, ParameterDefinition, ParameterExpansion, ParameterExpr,
};
use dokime_domain::operations::catalog as wire;
use dokime_domain::operations::common;
use dokime_domain::operations::schema;
use dokime_domain::operations::value as wire_value;

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
    let mut bytes = std::mem::size_of::<Vec<wire::ParameterSchema>>().saturating_add(
        parameters
            .len()
            .saturating_mul(std::mem::size_of::<wire::ParameterSchema>()),
    );
    for parameter in parameters {
        budget.node(0).map_err(|_| exhausted())?;
        bytes = bytes
            .saturating_add(parameter.parameter_id.as_str().len())
            .saturating_add(
                canonical::metadata_memory(&parameter.metadata).map_err(|_| exhausted())?
                    - std::mem::size_of::<common::Metadata>(),
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
            bytes = bytes.saturating_add(
                unit_memory(source)? - std::mem::size_of::<schema::MeasurementUnit>(),
            );
        }
        value::collection(parameter.extensions.len()).map_err(|_| exhausted())?;
        bytes = bytes.saturating_add(
            parameter
                .extensions
                .len()
                .saturating_mul(std::mem::size_of::<wire::ParameterSchemaExtensionsEntry>()),
        );
        for (key, item) in &parameter.extensions {
            value::text(key).map_err(|_| exhausted())?;
            bytes = bytes.saturating_add(key.len()).saturating_add(
                canonical::memory(item, budget, 0).map_err(|_| exhausted())?
                    - std::mem::size_of::<wire_value::Value>(),
            );
        }
        if let Some(selection) = &parameter.selection {
            value::collection(selection.choices.len()).map_err(|_| exhausted())?;
            for choice in &selection.choices {
                bytes = bytes
                    .saturating_add(canonical::memory(choice, budget, 0).map_err(|_| exhausted())?);
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
    let metadata = canonical::metadata_memory(&source.metadata).map_err(|_| exhausted())?
        - std::mem::size_of::<common::Metadata>();
    std::mem::size_of::<schema::MeasurementUnit>()
        .checked_add(source.symbol.len())
        .and_then(|bytes| bytes.checked_add(metadata))
        .ok_or_else(exhausted)
}

pub(in crate::grpc) fn wire(
    parameters: &[ParameterDefinition],
    budget: &mut Budget,
) -> Result<Vec<wire::ParameterSchema>, Status> {
    value::collection(parameters.len())?;
    collect(parameters.iter().map(|parameter| {
        budget.node(0)?;
        value::collection(parameter.extensions.len())?;
        let mut extensions = canonical::slots(parameter.extensions.len())?;
        for (key, item) in &parameter.extensions {
            value::text(key)?;
            extensions.push(wire::ParameterSchemaExtensionsEntry {
                key: key.clone(),
                value: Some(project_value(item, budget, 0)?),
            });
        }
        let selection = parameter
            .selection
            .as_ref()
            .map(|selection| {
                value::collection(selection.choices.len())?;
                Ok::<_, Status>(wire::ParameterSelection {
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
            dokime_runtime::definition::ParameterExpansion::Single => {
                wire::ParameterExpansionKind::Single(true)
            }
            dokime_runtime::definition::ParameterExpansion::Cartesian => {
                wire::ParameterExpansionKind::Cartesian(true)
            }
            dokime_runtime::definition::ParameterExpansion::Zip { group } => {
                wire::ParameterExpansionKind::Zip(group.as_str().to_owned())
            }
        };
        Ok(wire::ParameterSchema {
            expansion: Some(wire::ParameterExpansion { kind: Some(kind) }),
            parameter_id: parameter.parameter_id.as_str().to_owned(),
            metadata: Some(metadata(&parameter.metadata, false)?),
            group_id: parameter.group_id.as_ref().map(|id| id.as_str().to_owned()),
            value_type: canonical::value_type(&parameter.value_type),
            unit: parameter.unit.as_ref().map(unit).transpose()?,
            required: parameter.required,
            runtime_writable: parameter.runtime_writable,
            min: parameter.min,
            max: parameter.max,
            expression: Some(expression(&parameter.expression, budget, 0)?),
            selection,
            list: parameter.list.as_ref().map(|list| wire::ParameterList {
                element_type: canonical::value_type(&list.element_type),
                min_items: list.min_items.map(|items| items as u32),
                max_items: list.max_items.map(|items| items as u32),
            }),
            extensions,
        })
    }))
}

pub(in crate::grpc) fn unit(unit: &MeasurementUnit) -> Result<schema::MeasurementUnit, Status> {
    value::text(&unit.symbol)?;
    Ok(schema::MeasurementUnit {
        metadata: Some(metadata(&unit.metadata, true)?),
        symbol: unit.symbol.clone(),
        family: match unit.family {
            UnitFamily::Count => schema::UnitFamily::Count,
            UnitFamily::Distance => schema::UnitFamily::Distance,
            UnitFamily::Duration => schema::UnitFamily::Duration,
            UnitFamily::Frequency => schema::UnitFamily::Frequency,
            UnitFamily::Power => schema::UnitFamily::Power,
            UnitFamily::Temperature => schema::UnitFamily::Temperature,
            UnitFamily::Voltage => schema::UnitFamily::Voltage,
            UnitFamily::Ratio => schema::UnitFamily::Ratio,
            UnitFamily::Custom => schema::UnitFamily::Custom,
        },
    })
}

/* === Private Functions === */
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
) -> Result<wire::ParameterExpression, Status> {
    let mut nodes = Vec::new();
    flatten(expr, budget, depth, &mut nodes)?;
    Ok(wire::ParameterExpression { nodes })
}

fn flatten(
    expr: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
    nodes: &mut Vec<wire::ParameterNode>,
) -> Result<u32, Status> {
    use wire::ParameterNodeKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len())
        .map_err(|_| Status::resource_exhausted("expression nodes overflow"))?;
    nodes.push(wire::ParameterNode { kind: None });
    let kind = match expr {
        ParameterExpr::Literal { value } => Kind::Literal(project_value(value, budget, depth + 1)?),
        ParameterExpr::Array { values } => {
            let mut operands = canonical::slots(values.len())?;
            for value in values {
                operands.push(flatten(value, budget, depth + 1, nodes)?);
            }
            Kind::Array(wire::ParameterOperands { values: operands })
        }
        ParameterExpr::Range { start, stop, step } => Kind::Range(wire::IntegerRange {
            start: *start,
            stop: *stop,
            step: *step,
        }),
        ParameterExpr::Arange { start, stop, step } => {
            finite(&[*start, *stop, *step])?;
            Kind::Arange(wire::NumberRange {
                start: *start,
                stop: *stop,
                step: *step,
            })
        }
        ParameterExpr::Linspace { start, stop, count } => {
            finite(&[*start, *stop])?;
            Kind::Linspace(wire::LinearSpace {
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
            Kind::DynamicDefault(wire::DynamicDefault {
                key: match key {
                    DynamicDefaultKey::OperatorName => wire::DynamicDefaultKey::OperatorName,
                    DynamicDefaultKey::OperatorEmail => wire::DynamicDefaultKey::OperatorEmail,
                },
                fallback: fallback
                    .as_ref()
                    .map(|expr| flatten(expr, budget, depth + 1, nodes))
                    .transpose()?,
            })
        }
        ParameterExpr::Unary { value, .. } => Kind::Unary(wire::UnaryExpression {
            op: wire::UnaryOperator::Negate,
            value: flatten(value, budget, depth + 1, nodes)?,
        }),
        ParameterExpr::Binary { op, left, right } => Kind::Binary(wire::BinaryExpression {
            op: match op {
                ParameterBinaryOp::Add => wire::BinaryOperator::Add,
                ParameterBinaryOp::Subtract => wire::BinaryOperator::Subtract,
                ParameterBinaryOp::Multiply => wire::BinaryOperator::Multiply,
                ParameterBinaryOp::Divide => wire::BinaryOperator::Divide,
            },
            left: flatten(left, budget, depth + 1, nodes)?,
            right: flatten(right, budget, depth + 1, nodes)?,
        }),
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}

fn expression_memory(
    expr: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
) -> harmos::Result<usize> {
    budget.node(depth).map_err(|_| exhausted())?;
    // Includes Conservative Capacity For The Flat Node Vector, Not Nested Box Allocations.
    let mut bytes = 4usize.saturating_mul(std::mem::size_of::<wire::ParameterNode>());
    bytes = bytes.saturating_add(match expr {
        ParameterExpr::Literal { value } => {
            canonical::memory(value, budget, depth + 1).map_err(|_| exhausted())?
                - std::mem::size_of::<wire_value::Value>()
        }
        ParameterExpr::Array { values } => {
            value::collection(values.len()).map_err(|_| exhausted())?;
            let mut bytes = values.len().saturating_mul(std::mem::size_of::<u32>());
            for value in values {
                bytes = bytes.saturating_add(expression_memory(value, budget, depth + 1)?);
            }
            bytes
        }
        ParameterExpr::Reference { parameter } => {
            value::text(parameter).map_err(|_| exhausted())?;
            parameter.len()
        }
        ParameterExpr::DynamicDefault { fallback, .. } => fallback
            .as_ref()
            .map(|expr| expression_memory(expr, budget, depth + 1))
            .transpose()?
            .unwrap_or(0),
        ParameterExpr::Unary { value, .. } => expression_memory(value, budget, depth + 1)?,
        ParameterExpr::Binary { left, right, .. } => expression_memory(left, budget, depth + 1)?
            .saturating_add(expression_memory(right, budget, depth + 1)?),
        ParameterExpr::Range { .. }
        | ParameterExpr::Arange { .. }
        | ParameterExpr::Linspace { .. } => 0,
    });
    if bytes == usize::MAX {
        Err(exhausted())
    } else {
        Ok(bytes)
    }
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical parameter construction",
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = canonical::slots(source.len())?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata(source: &dokime_domain::model::identity::Metadata, _: bool) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime_domain::model::values::Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<wire_value::Value, Status> {
    canonical::encode(source, budget, depth)
}

/* === Tests === */

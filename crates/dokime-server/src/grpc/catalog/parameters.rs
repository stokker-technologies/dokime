/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Parameter Projection And Value Validation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tonic::Status;

// Local Imports
use super::super::value::Budget;
use super::super::value::canonical;
use dokime::definition::{DynamicDefaultKey, ParameterBinaryOp, ParameterDefinition, ParameterExpr};
use dokime::domain::model::definition::wire as schema;
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::values::MeasurementUnit;
use dokime::domain::model::values::UnitFamily;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn wire(
    parameters: &[ParameterDefinition], budget: &mut Budget,
) -> Result<Vec<dokime::domain::model::catalog::wire::ParameterSchema>, Status> {
    collect(parameters.iter().map(|parameter| {
        budget.node(0)?;
        let mut extensions = canonical::slots(parameter.extensions.len())?;
        for (key, item) in &parameter.extensions {
            extensions.push(dokime::domain::model::catalog::wire::ParameterSchemaExtensionsEntry {
                key: key.clone(),
                value: Some(project_value(item, budget, 0)?),
            });
        }
        let selection = parameter
            .selection
            .as_ref()
            .map(|selection| {
                Ok::<_, Status>(dokime::domain::model::catalog::wire::ParameterSelection {
                    choices: collect(selection.choices.iter().map(|value| project_value(value, budget, 0)))?,
                })
            })
            .transpose()?;
        let kind = match &parameter.expansion {
            dokime::definition::ParameterExpansion::Single => dokime::domain::model::catalog::wire::ParameterExpansionKind::Single(true),
            dokime::definition::ParameterExpansion::Cartesian => {
                dokime::domain::model::catalog::wire::ParameterExpansionKind::Cartesian(true)
            }
            dokime::definition::ParameterExpansion::Zip { group } => {
                dokime::domain::model::catalog::wire::ParameterExpansionKind::Zip(group.as_str().to_owned())
            }
        };
        Ok(dokime::domain::model::catalog::wire::ParameterSchema {
            expansion: Some(dokime::domain::model::catalog::wire::ParameterExpansion { kind: Some(kind) }),
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
            list: parameter.list.as_ref().map(|list| dokime::domain::model::catalog::wire::ParameterList {
                element_type: canonical::value_type(&list.element_type),
                min_items: list.min_items.map(|items| items as u32),
                max_items: list.max_items.map(|items| items as u32),
            }),
            extensions,
        })
    }))
}

pub(in crate::grpc) fn unit(unit: &MeasurementUnit) -> Result<schema::MeasurementUnit, Status> {
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
    if values.iter().any(|v| !v.is_finite()) { Err(Status::internal("catalog contains nonfinite expression")) } else { Ok(()) }
}

fn expression(
    expr: &ParameterExpr, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::catalog::wire::ParameterExpression, Status> {
    let mut nodes = Vec::new();
    flatten(expr, budget, depth, &mut nodes)?;
    Ok(dokime::domain::model::catalog::wire::ParameterExpression { nodes })
}

fn flatten(
    expr: &ParameterExpr, budget: &mut Budget, depth: usize, nodes: &mut Vec<dokime::domain::model::catalog::wire::ParameterNode>,
) -> Result<u32, Status> {
    use dokime::domain::model::catalog::wire::ParameterNodeKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len()).map_err(|_| Status::resource_exhausted("expression nodes overflow"))?;
    nodes.push(dokime::domain::model::catalog::wire::ParameterNode { kind: None });
    let kind = match expr {
        ParameterExpr::Literal { value } => Kind::Literal(project_value(value, budget, depth + 1)?),
        ParameterExpr::Array { values } => {
            let mut operands = canonical::slots(values.len())?;
            for value in values {
                operands.push(flatten(value, budget, depth + 1, nodes)?);
            }
            Kind::Array(dokime::domain::model::catalog::wire::ParameterOperands { values: operands })
        }
        ParameterExpr::Range { start, stop, step } => {
            Kind::Range(dokime::domain::model::catalog::wire::IntegerRange { start: *start, stop: *stop, step: *step })
        }
        ParameterExpr::Arange { start, stop, step } => {
            finite(&[*start, *stop, *step])?;
            Kind::Arange(dokime::domain::model::catalog::wire::NumberRange { start: *start, stop: *stop, step: *step })
        }
        ParameterExpr::Linspace { start, stop, count } => {
            finite(&[*start, *stop])?;
            Kind::Linspace(dokime::domain::model::catalog::wire::LinearSpace { start: *start, stop: *stop, count: *count as u64 })
        }
        ParameterExpr::Reference { parameter } => Kind::Reference(parameter.clone()),
        ParameterExpr::DynamicDefault { key, fallback } => Kind::DynamicDefault(dokime::domain::model::catalog::wire::DynamicDefault {
            key: match key {
                DynamicDefaultKey::OperatorName => dokime::domain::model::catalog::wire::DynamicDefaultKey::OperatorName,
                DynamicDefaultKey::OperatorEmail => dokime::domain::model::catalog::wire::DynamicDefaultKey::OperatorEmail,
            },
            fallback: fallback.as_ref().map(|expr| flatten(expr, budget, depth + 1, nodes)).transpose()?,
        }),
        ParameterExpr::Unary { value, .. } => Kind::Unary(dokime::domain::model::catalog::wire::UnaryExpression {
            op: dokime::domain::model::catalog::wire::UnaryOperator::Negate,
            value: flatten(value, budget, depth + 1, nodes)?,
        }),
        ParameterExpr::Binary { op, left, right } => Kind::Binary(dokime::domain::model::catalog::wire::BinaryExpression {
            op: match op {
                ParameterBinaryOp::Add => dokime::domain::model::catalog::wire::BinaryOperator::Add,
                ParameterBinaryOp::Subtract => dokime::domain::model::catalog::wire::BinaryOperator::Subtract,
                ParameterBinaryOp::Multiply => dokime::domain::model::catalog::wire::BinaryOperator::Multiply,
                ParameterBinaryOp::Divide => dokime::domain::model::catalog::wire::BinaryOperator::Divide,
            },
            left: flatten(left, budget, depth + 1, nodes)?,
            right: flatten(right, budget, depth + 1, nodes)?,
        }),
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = canonical::slots(source.len())?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata(source: &dokime::domain::model::identity::Metadata, _: bool) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime::domain::model::values::Value, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

/* === Tests === */

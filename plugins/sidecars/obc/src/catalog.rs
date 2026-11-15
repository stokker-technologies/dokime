/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Declaration Helpers Shared by Every Suite

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{
    Card, CardLayout, DataExpression, EvidenceCard, FieldReference, Metadata, Parameter, ParameterExpansion, ParameterExpression,
    ParameterExpressions, Plot, PlotKind, Value, ValueList, ValueType, card, parameter_expansion, parameter_expression, value,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// One Field of a Declared Stream.
pub fn field(stream: &str, name: &str) -> DataExpression {
    FieldReference::new(stream).field(name)
}

/// A Plot With Its Identity, Label and Unit; Callers Fill the Axes.
pub fn plot(id: impl Into<String>, label: impl Into<String>, kind: PlotKind, unit: &str) -> Plot {
    Plot {
        metadata: Some(metadata(id, label)),
        kind: kind as i32,
        unit: unit.into(),
        ..Default::default()
    }
}

/// A Plot Card Spanning the Given Number of Layout Columns.
pub fn wide(plot: Plot, columns: u32) -> Card {
    Card {
        metadata: plot.metadata.clone(),
        body: Some(card::Body::Plot(plot)),
        layout: Some(CardLayout { columns, ..Default::default() }),
        ..Default::default()
    }
}

/// The Records Captured by One Execution.
pub fn records(id: &str) -> Card {
    Card {
        metadata: Some(metadata(id, "Records")),
        body: Some(card::Body::Evidence(EvidenceCard::Records as i32)),
        ..Default::default()
    }
}

pub fn metadata(id: impl Into<String>, label: impl Into<String>) -> Metadata {
    Metadata {
        identifier: id.into(),
        label: label.into(),
        ..Default::default()
    }
}

pub fn number(id: &str, label: &str, unit: &str, default: f64, min: Option<f64>, max: Option<f64>) -> Parameter {
    Parameter {
        unit: unit.into(),
        min,
        max,
        ..literal(id, label, ValueType::Number, value::Kind::Number(default))
    }
}

pub fn count(id: &str, label: &str, default: u64, min: f64, max: f64) -> Parameter {
    Parameter {
        min: Some(min),
        max: Some(max),
        ..literal(id, label, ValueType::UnsignedInteger, value::Kind::UnsignedInteger(default))
    }
}

pub fn flag(id: &str, label: &str, default: bool) -> Parameter {
    literal(id, label, ValueType::Boolean, value::Kind::Boolean(default))
}

/// A Number Swept Across Cases: Each Case Receives One of the Listed Values.
pub fn sweep(id: &str, label: &str, unit: &str, values: &[f64]) -> Parameter {
    let values = values
        .iter()
        .map(|number| ParameterExpression {
            kind: Some(parameter_expression::Kind::Literal(Value { kind: Some(value::Kind::Number(*number)) })),
        })
        .collect();
    Parameter {
        metadata: Some(metadata(id, label)),
        r#type: ValueType::Number as i32,
        unit: unit.into(),
        expression: Some(ParameterExpression {
            kind: Some(parameter_expression::Kind::Array(ParameterExpressions { values })),
        }),
        expansion: Some(ParameterExpansion {
            kind: Some(parameter_expansion::Kind::Cartesian(true)),
        }),
        ..Default::default()
    }
}

/// An Ordered List of Numbers One Case Replays in Full.
pub fn numbers(id: &str, label: &str, unit: &str, values: &[f64]) -> Parameter {
    let values = values.iter().map(|number| Value { kind: Some(value::Kind::Number(*number)) }).collect();
    Parameter {
        unit: unit.into(),
        element_type: ValueType::Number as i32,
        min_items: Some(1),
        max_items: Some(16),
        ..literal(id, label, ValueType::List, value::Kind::List(ValueList { values }))
    }
}

/* === Private Functions === */

fn literal(id: &str, label: &str, kind: ValueType, default: value::Kind) -> Parameter {
    Parameter {
        metadata: Some(metadata(id, label)),
        r#type: kind as i32,
        expression: Some(ParameterExpression {
            kind: Some(parameter_expression::Kind::Literal(Value { kind: Some(default) })),
        }),
        ..Default::default()
    }
}

/* === Tests === */

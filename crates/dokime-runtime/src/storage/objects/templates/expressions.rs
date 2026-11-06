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
use serde::{Deserialize, Serialize};

// Local Imports
use super::Value;
use crate::definition::{DynamicDefaultKey, ParameterBinaryOp, ParameterExpr, ParameterUnaryOp};
use crate::expressions::{BooleanExpression, Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp};
use crate::storage::{Error, Result};

/* === Constants === */
const MAX_DEPTH: usize = 64;

/* === Macros === */
macro_rules! serde_expression {
    ($row:ident, $domain:ty) => {
        impl Serialize for $row {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
                let value: $domain = self.clone().try_into().map_err(serde::ser::Error::custom)?;
                value.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $row {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                let value = <$domain>::deserialize(deserializer)?;
                Self::try_from(&value).map_err(serde::de::Error::custom)
            }
        }
    };
}
serde_expression!(Parameter, crate::definition::ParameterExpr);
serde_expression!(Boolean, crate::expressions::BooleanExpression);

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct Parameter {
    nodes: Vec<Node>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct Boolean {
    nodes: Vec<Node>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Node {
    kind: String,
    children: u32,
    literal: Option<Value>,
    reference: Option<String>,
    key: Option<String>,
    op: Option<String>,
    path: Option<String>,
    function: Option<String>,
    integer_start: Option<i64>,
    integer_stop: Option<i64>,
    integer_step: Option<i64>,
    number_start: Option<f64>,
    number_stop: Option<f64>,
    number_step: Option<f64>,
    count: Option<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TryFrom<&BooleanExpression> for Boolean {
    type Error = Error;

    fn try_from(value: &BooleanExpression) -> Result<Self> {
        let mut nodes = Vec::new();
        append_boolean(&value.expression, &mut nodes, 0)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Boolean> for BooleanExpression {
    type Error = Error;

    fn try_from(row: Boolean) -> Result<Self> {
        let mut nodes = row.nodes.into_iter();
        let expression = consume_boolean(&mut nodes, 0)?;
        no_trailing(&nodes)?;
        Ok(Self { expression })
    }
}

impl TryFrom<&ParameterExpr> for Parameter {
    type Error = Error;

    fn try_from(value: &ParameterExpr) -> Result<Self> {
        let mut nodes = Vec::new();
        append_parameter(value, &mut nodes, 0)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Parameter> for ParameterExpr {
    type Error = Error;

    fn try_from(row: Parameter) -> Result<Self> {
        let mut nodes = row.nodes.into_iter();
        let expression = consume_parameter(&mut nodes, 0)?;
        no_trailing(&nodes)?;
        Ok(expression)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn admit_depth(depth: usize) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("expression exceeds depth limit"));
    }
    Ok(())
}

fn admit_children(node: &Node, remaining: &std::vec::IntoIter<Node>) -> Result<()> {
    if node.children as usize > remaining.len() {
        return Err(Error::Invalid("expression children exceed remaining nodes"));
    }
    Ok(())
}

fn no_trailing(remaining: &std::vec::IntoIter<Node>) -> Result<()> {
    if remaining.len() > 0 {
        return Err(Error::Invalid("expression has trailing nodes"));
    }
    Ok(())
}

fn child_count(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| Error::Invalid("too many expression children"))
}

fn append_parameter(value: &ParameterExpr, nodes: &mut Vec<Node>, depth: usize) -> Result<()> {
    admit_depth(depth)?;
    let mut node = Node::default();
    let kind = match value {
        ParameterExpr::Literal { value } => {
            node.literal = Some(Value::try_from(value)?);
            "literal"
        }
        ParameterExpr::Array { values } => {
            node.children = child_count(values.len())?;
            "array"
        }
        ParameterExpr::Range { start, stop, step } => {
            node.integer_start = Some(*start);
            node.integer_stop = Some(*stop);
            node.integer_step = Some(*step);
            "range"
        }
        ParameterExpr::Arange { start, stop, step } => {
            if ![start, stop, step].into_iter().all(|value| value.is_finite()) {
                return Err(Error::Invalid("parameter expression number is nonfinite"));
            }
            node.number_start = Some(*start);
            node.number_stop = Some(*stop);
            node.number_step = Some(*step);
            "arange"
        }
        ParameterExpr::Linspace { start, stop, count } => {
            if ![start, stop].into_iter().all(|value| value.is_finite()) {
                return Err(Error::Invalid("parameter expression number is nonfinite"));
            }
            node.number_start = Some(*start);
            node.number_stop = Some(*stop);
            node.count = Some(u64::try_from(*count).map_err(|_| Error::Invalid("expression count overflows"))?);
            "linspace"
        }
        ParameterExpr::Reference { parameter } => {
            node.reference = Some(parameter.as_str().to_owned());
            "reference"
        }
        ParameterExpr::DynamicDefault { key, fallback } => {
            node.key = Some(dynamic_key_name(key).to_owned());
            node.children = u32::from(fallback.is_some());
            "dynamic_default"
        }
        ParameterExpr::Unary { .. } => {
            node.op = Some("negate".into());
            node.children = 1;
            "unary"
        }
        ParameterExpr::Binary { op, .. } => {
            node.op = Some(parameter_op_name(op).to_owned());
            node.children = 2;
            "binary"
        }
    };
    node.kind = kind.to_owned();
    nodes.push(node);
    match value {
        ParameterExpr::Array { values } => {
            for child in values {
                append_parameter(child, nodes, depth + 1)?;
            }
        }
        ParameterExpr::DynamicDefault { fallback: Some(child), .. } | ParameterExpr::Unary { value: child, .. } => {
            append_parameter(child, nodes, depth + 1)?
        }
        ParameterExpr::Binary { left, right, .. } => {
            append_parameter(left, nodes, depth + 1)?;
            append_parameter(right, nodes, depth + 1)?;
        }
        _ => {}
    }
    Ok(())
}

fn append_boolean(value: &Expression, nodes: &mut Vec<Node>, depth: usize) -> Result<()> {
    admit_depth(depth)?;
    let mut node = Node::default();
    let kind = match &value.kind {
        ExpressionKind::Literal { value } => {
            node.literal = Some(Value::try_from(value)?);
            "literal"
        }
        ExpressionKind::Selector { path } => {
            node.path = Some(path.as_str().to_owned());
            "selector"
        }
        ExpressionKind::Unary { .. } => {
            node.op = Some("not".into());
            node.children = 1;
            "unary"
        }
        ExpressionKind::Binary { op, .. } => {
            node.op = Some(boolean_op_name(op).to_owned());
            node.children = 2;
            "binary"
        }
        ExpressionKind::Call { function, args } => {
            node.function = Some(function.as_str().to_owned());
            node.children = child_count(args.len())?;
            "call"
        }
    };
    node.kind = kind.to_owned();
    nodes.push(node);
    match &value.kind {
        ExpressionKind::Unary { value, .. } => append_boolean(value, nodes, depth + 1)?,
        ExpressionKind::Binary { left, right, .. } => {
            append_boolean(left, nodes, depth + 1)?;
            append_boolean(right, nodes, depth + 1)?;
        }
        ExpressionKind::Call { args, .. } => {
            for child in args {
                append_boolean(child, nodes, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn dynamic_key_name(key: &DynamicDefaultKey) -> &'static str {
    match key {
        DynamicDefaultKey::OperatorName => "operator_name",
        DynamicDefaultKey::OperatorEmail => "operator_email",
    }
}

fn parameter_op_name(op: &ParameterBinaryOp) -> &'static str {
    match op {
        ParameterBinaryOp::Add => "add",
        ParameterBinaryOp::Subtract => "subtract",
        ParameterBinaryOp::Multiply => "multiply",
        ParameterBinaryOp::Divide => "divide",
    }
}

fn boolean_op_name(op: &ExpressionBinaryOp) -> &'static str {
    match op {
        ExpressionBinaryOp::And => "and",
        ExpressionBinaryOp::Or => "or",
        ExpressionBinaryOp::Eq => "eq",
        ExpressionBinaryOp::Ne => "ne",
        ExpressionBinaryOp::Gt => "gt",
        ExpressionBinaryOp::Gte => "gte",
        ExpressionBinaryOp::Lt => "lt",
        ExpressionBinaryOp::Lte => "lte",
        ExpressionBinaryOp::Contains => "contains",
        ExpressionBinaryOp::StartsWith => "starts_with",
        ExpressionBinaryOp::EndsWith => "ends_with",
    }
}

fn validate_boolean(node: &Node) -> Result<()> {
    let kind = node.kind.as_str();
    let arity = match kind {
        "literal" | "selector" => node.children == 0,
        "unary" => node.children == 1,
        "binary" => node.children == 2,
        "call" => true,
        _ => false,
    };
    let fields = [
        (node.literal.is_some(), kind == "literal"),
        (node.path.is_some(), kind == "selector"),
        (node.function.is_some(), kind == "call"),
        (node.op.is_some(), matches!(kind, "unary" | "binary")),
        (node.reference.is_some(), false),
        (node.key.is_some(), false),
        (node.integer_start.is_some(), false),
        (node.integer_stop.is_some(), false),
        (node.integer_step.is_some(), false),
        (node.number_start.is_some(), false),
        (node.number_stop.is_some(), false),
        (node.number_step.is_some(), false),
        (node.count.is_some(), false),
    ];
    if !arity || fields.iter().any(|(present, required)| present != required) {
        return Err(Error::Invalid("Boolean expression fields or arity differ"));
    }
    match kind {
        "unary" if node.op.as_deref() != Some("not") => {
            return Err(Error::Invalid("invalid Boolean unary operator"));
        }
        "binary" => {
            boolean_binary(node.op.as_deref().unwrap())?;
        }
        _ => {}
    }
    Ok(())
}

fn consume_boolean(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<Expression> {
    admit_depth(depth)?;
    let node = nodes.next().ok_or(Error::Invalid("expression child is missing"))?;
    validate_boolean(&node)?;
    admit_children(&node, nodes)?;
    let kind = match node.kind.as_str() {
        "literal" => ExpressionKind::Literal { value: node.literal.unwrap().try_into()? },
        "selector" => ExpressionKind::Selector { path: node.path.unwrap() },
        "unary" => ExpressionKind::Unary {
            op: ExpressionUnaryOp::Not,
            value: Box::new(consume_boolean(nodes, depth + 1)?),
        },
        "binary" => {
            let op = boolean_binary(node.op.as_deref().unwrap())?;
            let left = Box::new(consume_boolean(nodes, depth + 1)?);
            let right = Box::new(consume_boolean(nodes, depth + 1)?);
            ExpressionKind::Binary { op, left, right }
        }
        "call" => {
            let mut args = Vec::new();
            args.try_reserve_exact(node.children as usize).map_err(|_| Error::Invalid("expression argument allocation failed"))?;
            for _ in 0..node.children {
                args.push(consume_boolean(nodes, depth + 1)?);
            }
            ExpressionKind::Call { function: node.function.unwrap(), args }
        }
        _ => return Err(Error::Invalid("unknown Boolean expression kind")),
    };
    Ok(Expression { kind })
}

fn boolean_binary(op: &str) -> Result<ExpressionBinaryOp> {
    match op {
        "and" => Ok(ExpressionBinaryOp::And),
        "or" => Ok(ExpressionBinaryOp::Or),
        "eq" => Ok(ExpressionBinaryOp::Eq),
        "ne" => Ok(ExpressionBinaryOp::Ne),
        "gt" => Ok(ExpressionBinaryOp::Gt),
        "gte" => Ok(ExpressionBinaryOp::Gte),
        "lt" => Ok(ExpressionBinaryOp::Lt),
        "lte" => Ok(ExpressionBinaryOp::Lte),
        "contains" => Ok(ExpressionBinaryOp::Contains),
        "starts_with" => Ok(ExpressionBinaryOp::StartsWith),
        "ends_with" => Ok(ExpressionBinaryOp::EndsWith),
        _ => Err(Error::Invalid("invalid Boolean binary operator")),
    }
}

// A Node May Carry Only The Fields Of Its Selected Variant.
fn validate_parameter(node: &Node) -> Result<()> {
    let arity = match node.kind.as_str() {
        "literal" | "reference" | "range" | "arange" | "linspace" => node.children == 0,
        "array" => true,
        "dynamic_default" => node.children <= 1,
        "unary" => node.children == 1,
        "binary" => node.children == 2,
        _ => false,
    };
    let kind = node.kind.as_str();
    let fields = [
        (node.literal.is_some(), kind == "literal"),
        (node.reference.is_some(), kind == "reference"),
        (node.key.is_some(), kind == "dynamic_default"),
        (node.op.is_some(), matches!(kind, "unary" | "binary")),
        (node.path.is_some(), false),
        (node.function.is_some(), false),
        (node.integer_start.is_some(), kind == "range"),
        (node.integer_stop.is_some(), kind == "range"),
        (node.integer_step.is_some(), kind == "range"),
        (node.number_start.is_some(), matches!(kind, "arange" | "linspace")),
        (node.number_stop.is_some(), matches!(kind, "arange" | "linspace")),
        (node.number_step.is_some(), kind == "arange"),
        (node.count.is_some(), kind == "linspace"),
    ];
    if !arity || fields.iter().any(|(present, required)| present != required) {
        return Err(Error::Invalid("parameter expression fields or arity differ"));
    }
    if [node.number_start, node.number_stop, node.number_step].into_iter().flatten().any(|value| !value.is_finite()) {
        return Err(Error::Invalid("parameter expression number is nonfinite"));
    }
    if let Some(count) = node.count {
        usize::try_from(count).map_err(|_| Error::Invalid("expression count overflows"))?;
    }
    match kind {
        "dynamic_default" => {
            dynamic_key(node.key.as_deref().unwrap())?;
        }
        "unary" if node.op.as_deref() != Some("negate") => {
            return Err(Error::Invalid("invalid parameter unary operator"));
        }
        "binary" => {
            parameter_binary(node.op.as_deref().unwrap())?;
        }
        _ => {}
    }
    Ok(())
}

fn consume_parameter(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<ParameterExpr> {
    admit_depth(depth)?;
    let node = nodes.next().ok_or(Error::Invalid("expression child is missing"))?;
    validate_parameter(&node)?;
    admit_children(&node, nodes)?;
    Ok(match node.kind.as_str() {
        "literal" => ParameterExpr::Literal { value: node.literal.unwrap().try_into()? },
        "reference" => ParameterExpr::Reference { parameter: node.reference.unwrap() },
        "array" => {
            let mut values = Vec::new();
            values.try_reserve_exact(node.children as usize).map_err(|_| Error::Invalid("expression array allocation failed"))?;
            for _ in 0..node.children {
                values.push(consume_parameter(nodes, depth + 1)?);
            }
            ParameterExpr::Array { values }
        }
        "dynamic_default" => {
            let key = dynamic_key(node.key.as_deref().unwrap())?;
            let fallback = if node.children == 0 { None } else { Some(Box::new(consume_parameter(nodes, depth + 1)?)) };
            ParameterExpr::DynamicDefault { key, fallback }
        }
        "unary" => ParameterExpr::Unary {
            op: ParameterUnaryOp::Negate,
            value: Box::new(consume_parameter(nodes, depth + 1)?),
        },
        "binary" => {
            let op = parameter_binary(node.op.as_deref().unwrap())?;
            let left = Box::new(consume_parameter(nodes, depth + 1)?);
            let right = Box::new(consume_parameter(nodes, depth + 1)?);
            ParameterExpr::Binary { op, left, right }
        }
        "range" => ParameterExpr::Range {
            start: node.integer_start.unwrap(),
            stop: node.integer_stop.unwrap(),
            step: node.integer_step.unwrap(),
        },
        "arange" => ParameterExpr::Arange {
            start: node.number_start.unwrap(),
            stop: node.number_stop.unwrap(),
            step: node.number_step.unwrap(),
        },
        "linspace" => ParameterExpr::Linspace {
            start: node.number_start.unwrap(),
            stop: node.number_stop.unwrap(),
            count: usize::try_from(node.count.unwrap()).map_err(|_| Error::Invalid("expression count overflows"))?,
        },
        _ => return Err(Error::Invalid("unknown parameter expression kind")),
    })
}

fn dynamic_key(key: &str) -> Result<DynamicDefaultKey> {
    match key {
        "operator_name" => Ok(DynamicDefaultKey::OperatorName),
        "operator_email" => Ok(DynamicDefaultKey::OperatorEmail),
        _ => Err(Error::Invalid("invalid dynamic default key")),
    }
}

fn parameter_binary(op: &str) -> Result<ParameterBinaryOp> {
    match op {
        "add" => Ok(ParameterBinaryOp::Add),
        "subtract" => Ok(ParameterBinaryOp::Subtract),
        "multiply" => Ok(ParameterBinaryOp::Multiply),
        "divide" => Ok(ParameterBinaryOp::Divide),
        _ => Err(Error::Invalid("invalid parameter binary operator")),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::expressions::{BooleanExpression, Expression, ExpressionBinaryOp, ExpressionKind};

    fn boolean_tree(leaves: usize) -> Expression {
        if leaves == 1 {
            return Expression {
                kind: ExpressionKind::Selector { path: "value".into() },
            };
        }
        let left_leaves = leaves / 2;
        Expression {
            kind: ExpressionKind::Binary {
                op: ExpressionBinaryOp::And,
                left: Box::new(boolean_tree(left_leaves)),
                right: Box::new(boolean_tree(leaves - left_leaves)),
            },
        }
    }

    #[test]
    fn admits_parameter_and_boolean_expressions_above_the_former_node_limit() {
        let values = (0..4096).map(|index| ParameterExpr::Reference { parameter: format!("parameter_{index}") }).collect();
        let parameter = ParameterExpr::Array { values };
        let parameter_row = Parameter::try_from(&parameter).unwrap();
        assert_eq!(parameter_row.nodes.len(), 4097);
        let decoded_parameter = ParameterExpr::try_from(parameter_row).unwrap();
        assert_eq!(decoded_parameter, parameter);

        let boolean = BooleanExpression { expression: boolean_tree(2049) };
        let boolean_row = Boolean::try_from(&boolean).unwrap();
        assert_eq!(boolean_row.nodes.len(), 4097);
        let decoded_boolean = BooleanExpression::try_from(boolean_row).unwrap();
        assert_eq!(decoded_boolean, boolean);
    }
}

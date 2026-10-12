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
use crate::expressions::{
    BooleanExpression, Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp,
};
use crate::storage::{Error, Result};

/* === Constants === */
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 4096;

/* === Macros === */
macro_rules! serde_expression {
    ($row:ident, $domain:ty) => {
        impl Serialize for $row {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                let value: $domain = self.clone().try_into().map_err(serde::ser::Error::custom)?;
                value.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $row {
            fn deserialize<D: serde::Deserializer<'de>>(
                deserializer: D,
            ) -> std::result::Result<Self, D::Error> {
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
impl Boolean {
    pub(super) fn encoding_memory(value: &BooleanExpression) -> Result<usize> {
        let mut count = 0;
        let fields = boolean_encoding(&value.expression, 0, &mut count)?;
        Ok(std::mem::size_of::<Self>().saturating_add(fields))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        if self.nodes.is_empty() || self.nodes.len() > MAX_NODES {
            return Err(Error::Invalid("expression exceeds node limit or is empty"));
        }
        let mut at = 0;
        let bound = boolean_memory(&self.nodes, &mut at, 0)?;
        if at != self.nodes.len() {
            return Err(Error::Invalid("expression has trailing nodes"));
        }
        Ok(bound)
    }
}

impl TryFrom<&BooleanExpression> for Boolean {
    type Error = Error;

    fn try_from(value: &BooleanExpression) -> Result<Self> {
        let mut count = 0;
        let memory = boolean_encoding(&value.expression, 0, &mut count)?;
        if std::mem::size_of::<Self>().saturating_add(memory) == usize::MAX {
            return Err(Error::Invalid("expression encoding memory overflows"));
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(count)
            .map_err(|_| Error::Invalid("expression nodes allocation failed"))?;
        append_boolean(&value.expression, &mut nodes)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Boolean> for BooleanExpression {
    type Error = Error;

    fn try_from(row: Boolean) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("expression projection memory overflows"));
        }
        let mut nodes = row.nodes.into_iter();
        let expression = consume_boolean(&mut nodes, 0)?;
        Ok(Self { expression })
    }
}

impl Parameter {
    pub(super) fn encoding_memory(value: &ParameterExpr) -> Result<usize> {
        let mut count = 0;
        let fields = parameter_encoding(value, 0, &mut count)?;
        Ok(std::mem::size_of::<Self>().saturating_add(fields))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        if self.nodes.is_empty() || self.nodes.len() > MAX_NODES {
            return Err(Error::Invalid("expression exceeds node limit or is empty"));
        }
        let mut at = 0;
        let bound = parameter_memory(&self.nodes, &mut at, 0)?;
        if at != self.nodes.len() {
            return Err(Error::Invalid("expression has trailing nodes"));
        }
        Ok(bound)
    }
}

impl TryFrom<&ParameterExpr> for Parameter {
    type Error = Error;

    fn try_from(value: &ParameterExpr) -> Result<Self> {
        let mut count = 0;
        let memory = parameter_encoding(value, 0, &mut count)?;
        if std::mem::size_of::<Self>().saturating_add(memory) == usize::MAX {
            return Err(Error::Invalid("expression encoding memory overflows"));
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(count)
            .map_err(|_| Error::Invalid("expression nodes allocation failed"))?;
        append_parameter(value, &mut nodes)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Parameter> for ParameterExpr {
    type Error = Error;

    fn try_from(row: Parameter) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("expression projection memory overflows"));
        }
        let mut nodes = row.nodes.into_iter();
        consume_parameter(&mut nodes, 0)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn admit_node(depth: usize, count: &mut usize) -> Result<()> {
    if depth > MAX_DEPTH || *count >= MAX_NODES {
        return Err(Error::Invalid("expression exceeds node or depth limit"));
    }
    *count += 1;
    Ok(())
}

fn parameter_encoding(value: &ParameterExpr, depth: usize, count: &mut usize) -> Result<usize> {
    admit_node(depth, count)?;
    let mut bytes = std::mem::size_of::<Node>();
    let fields = match value {
        ParameterExpr::Literal { value } => {
            let memory = Value::encoding_memory(value)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            "literal"
                .len()
                .saturating_add(memory.saturating_sub(std::mem::size_of::<Value>()))
        }
        ParameterExpr::Array { values } => {
            u32::try_from(values.len())
                .map_err(|_| Error::Invalid("too many expression children"))?;
            for child in values {
                bytes = bytes.saturating_add(parameter_encoding(child, depth + 1, count)?);
            }
            "array".len()
        }
        ParameterExpr::Range { .. } => "range".len(),
        ParameterExpr::Arange { start, stop, step } => {
            if ![start, stop, step]
                .into_iter()
                .all(|value| value.is_finite())
            {
                return Err(Error::Invalid("parameter expression number is nonfinite"));
            }
            "arange".len()
        }
        ParameterExpr::Linspace {
            start,
            stop,
            count: items,
        } => {
            if ![start, stop].into_iter().all(|value| value.is_finite()) {
                return Err(Error::Invalid("parameter expression number is nonfinite"));
            }
            u64::try_from(*items).map_err(|_| Error::Invalid("expression count overflows"))?;
            "linspace".len()
        }
        ParameterExpr::Reference { parameter } => "reference".len().saturating_add(parameter.len()),
        ParameterExpr::DynamicDefault { key, fallback } => {
            if let Some(child) = fallback {
                bytes = bytes.saturating_add(parameter_encoding(child, depth + 1, count)?);
            }
            "dynamic_default"
                .len()
                .saturating_add(dynamic_key_name(key).len())
        }
        ParameterExpr::Unary { value, .. } => {
            bytes = bytes.saturating_add(parameter_encoding(value, depth + 1, count)?);
            "unary".len().saturating_add("negate".len())
        }
        ParameterExpr::Binary { op, left, right } => {
            bytes = bytes
                .saturating_add(parameter_encoding(left, depth + 1, count)?)
                .saturating_add(parameter_encoding(right, depth + 1, count)?);
            "binary".len().saturating_add(parameter_op_name(op).len())
        }
    };
    Ok(bytes.saturating_add(fields))
}

fn append_parameter(value: &ParameterExpr, nodes: &mut Vec<Node>) -> Result<()> {
    let mut node = Node::default();
    let kind = match value {
        ParameterExpr::Literal { value } => {
            node.literal = Some(Value::try_from(value)?);
            "literal"
        }
        ParameterExpr::Array { values } => {
            node.children = values.len() as u32;
            "array"
        }
        ParameterExpr::Range { start, stop, step } => {
            node.integer_start = Some(*start);
            node.integer_stop = Some(*stop);
            node.integer_step = Some(*step);
            "range"
        }
        ParameterExpr::Arange { start, stop, step } => {
            node.number_start = Some(*start);
            node.number_stop = Some(*stop);
            node.number_step = Some(*step);
            "arange"
        }
        ParameterExpr::Linspace { start, stop, count } => {
            node.number_start = Some(*start);
            node.number_stop = Some(*stop);
            node.count = Some(
                u64::try_from(*count).map_err(|_| Error::Invalid("expression count overflows"))?,
            );
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
                append_parameter(child, nodes)?;
            }
        }
        ParameterExpr::DynamicDefault {
            fallback: Some(child),
            ..
        }
        | ParameterExpr::Unary { value: child, .. } => append_parameter(child, nodes)?,
        ParameterExpr::Binary { left, right, .. } => {
            append_parameter(left, nodes)?;
            append_parameter(right, nodes)?;
        }
        _ => {}
    }
    Ok(())
}

fn boolean_encoding(value: &Expression, depth: usize, count: &mut usize) -> Result<usize> {
    admit_node(depth, count)?;
    let mut bytes = std::mem::size_of::<Node>();
    let fields = match &value.kind {
        ExpressionKind::Literal { value } => {
            let memory = Value::encoding_memory(value)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            "literal"
                .len()
                .saturating_add(memory.saturating_sub(std::mem::size_of::<Value>()))
        }
        ExpressionKind::Selector { path } => "selector".len().saturating_add(path.len()),
        ExpressionKind::Unary { value, .. } => {
            bytes = bytes.saturating_add(boolean_encoding(value, depth + 1, count)?);
            "unary".len().saturating_add("not".len())
        }
        ExpressionKind::Binary { op, left, right } => {
            bytes = bytes
                .saturating_add(boolean_encoding(left, depth + 1, count)?)
                .saturating_add(boolean_encoding(right, depth + 1, count)?);
            "binary".len().saturating_add(boolean_op_name(op).len())
        }
        ExpressionKind::Call { function, args } => {
            u32::try_from(args.len())
                .map_err(|_| Error::Invalid("too many expression children"))?;
            for child in args {
                bytes = bytes.saturating_add(boolean_encoding(child, depth + 1, count)?);
            }
            "call".len().saturating_add(function.len())
        }
    };
    Ok(bytes.saturating_add(fields))
}

fn append_boolean(value: &Expression, nodes: &mut Vec<Node>) -> Result<()> {
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
            node.children = args.len() as u32;
            "call"
        }
    };
    node.kind = kind.to_owned();
    nodes.push(node);
    match &value.kind {
        ExpressionKind::Unary { value, .. } => append_boolean(value, nodes)?,
        ExpressionKind::Binary { left, right, .. } => {
            append_boolean(left, nodes)?;
            append_boolean(right, nodes)?;
        }
        ExpressionKind::Call { args, .. } => {
            for child in args {
                append_boolean(child, nodes)?;
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

fn boolean_memory(nodes: &[Node], at: &mut usize, depth: usize) -> Result<usize> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("expression exceeds depth limit"));
    }
    let node = nodes
        .get(*at)
        .ok_or(Error::Invalid("expression child is missing"))?;
    *at += 1;
    validate_boolean(node)?;
    if node.children as usize > nodes.len().saturating_sub(*at) {
        return Err(Error::Invalid("expression children exceed remaining nodes"));
    }
    let mut bytes = std::mem::size_of::<Expression>();
    if let Some(literal) = &node.literal {
        bytes = bytes.saturating_add(literal.projection_memory()?);
    }
    if let Some(path) = &node.path {
        bytes = bytes.saturating_add(path.capacity());
    }
    if let Some(function) = &node.function {
        bytes = bytes.saturating_add(function.capacity());
    }
    for _ in 0..node.children {
        bytes = bytes.saturating_add(boolean_memory(nodes, at, depth + 1)?);
    }
    Ok(bytes)
}

fn consume_boolean(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<Expression> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("expression exceeds depth limit"));
    }
    let node = nodes
        .next()
        .ok_or(Error::Invalid("expression child is missing"))?;
    let kind = match node.kind.as_str() {
        "literal" => ExpressionKind::Literal {
            value: node.literal.unwrap().try_into()?,
        },
        "selector" => ExpressionKind::Selector {
            path: node.path.unwrap(),
        },
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
            args.try_reserve_exact(node.children as usize)
                .map_err(|_| Error::Invalid("expression argument allocation failed"))?;
            for _ in 0..node.children {
                args.push(consume_boolean(nodes, depth + 1)?);
            }
            ExpressionKind::Call {
                function: node.function.unwrap(),
                args,
            }
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
        (
            node.number_start.is_some(),
            matches!(kind, "arange" | "linspace"),
        ),
        (
            node.number_stop.is_some(),
            matches!(kind, "arange" | "linspace"),
        ),
        (node.number_step.is_some(), kind == "arange"),
        (node.count.is_some(), kind == "linspace"),
    ];
    if !arity || fields.iter().any(|(present, required)| present != required) {
        return Err(Error::Invalid(
            "parameter expression fields or arity differ",
        ));
    }
    if [node.number_start, node.number_stop, node.number_step]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite())
    {
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

fn parameter_memory(nodes: &[Node], at: &mut usize, depth: usize) -> Result<usize> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("expression exceeds depth limit"));
    }
    let node = nodes
        .get(*at)
        .ok_or(Error::Invalid("expression child is missing"))?;
    *at += 1;
    validate_parameter(node)?;
    if node.children as usize > nodes.len().saturating_sub(*at) {
        return Err(Error::Invalid("expression children exceed remaining nodes"));
    }
    let mut bytes = std::mem::size_of::<ParameterExpr>();
    if let Some(literal) = &node.literal {
        let bound = literal.projection_memory()?;
        bytes = bytes.saturating_add(bound);
    }
    if let Some(reference) = &node.reference {
        bytes = bytes.saturating_add(reference.capacity());
    }
    for _ in 0..node.children {
        let child = parameter_memory(nodes, at, depth + 1)?;
        bytes = bytes.saturating_add(child);
    }
    Ok(bytes)
}

fn consume_parameter(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<ParameterExpr> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("expression exceeds depth limit"));
    }
    let node = nodes
        .next()
        .ok_or(Error::Invalid("expression child is missing"))?;
    // The Entire Tree Has Already Passed Borrowed Validation.
    Ok(match node.kind.as_str() {
        "literal" => ParameterExpr::Literal {
            value: node.literal.unwrap().try_into()?,
        },
        "reference" => ParameterExpr::Reference {
            parameter: node.reference.unwrap(),
        },
        "array" => {
            let mut values = Vec::new();
            values
                .try_reserve_exact(node.children as usize)
                .map_err(|_| Error::Invalid("expression array allocation failed"))?;
            for _ in 0..node.children {
                values.push(consume_parameter(nodes, depth + 1)?);
            }
            ParameterExpr::Array { values }
        }
        "dynamic_default" => {
            let key = dynamic_key(node.key.as_deref().unwrap())?;
            let fallback = if node.children == 0 {
                None
            } else {
                Some(Box::new(consume_parameter(nodes, depth + 1)?))
            };
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
            count: usize::try_from(node.count.unwrap())
                .map_err(|_| Error::Invalid("expression count overflows"))?,
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

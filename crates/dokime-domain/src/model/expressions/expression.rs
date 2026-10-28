/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::model::values::Value;
use crate::model::values::numeric_cmp;

product_error!(ExpressionError, formatter {
    Self::UnknownSelector(selector) => write!(formatter, "unknown selector {selector}"),
    Self::ExpectedBoolean => formatter.write_str("expression did not evaluate to boolean"),
    Self::ExpectedText => formatter.write_str("expression did not evaluate to text"),
    Self::ExpectedNumber => formatter.write_str("expression did not evaluate to number"),
    Self::UnsupportedCall(call) => write!(formatter, "unsupported expression call {call}"),
});

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExpressionContext {
    #[serde(default)]
    pub values: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    #[serde(flatten)]
    pub kind: ExpressionKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BooleanExpression {
    pub expression: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BooleanExpressionBuilder {
    left: Expression,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ExpressionKind {
    Literal {
        value: Value,
    },
    Selector {
        path: String,
    },
    Unary {
        op: ExpressionUnaryOp,
        value: Box<Expression>,
    },
    Binary {
        op: ExpressionBinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        function: String,

        #[serde(default)]
        args: Vec<Expression>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionUnaryOp {
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionBinaryOp {
    And,
    Or,
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionError {
    UnknownSelector(String),
    ExpectedBoolean,
    ExpectedText,
    ExpectedNumber,
    UnsupportedCall(String),
}

impl harmos_domain::MemorySize for Expression {
    fn memory_size(&self) -> usize {
        expression_memory(self, 0)
    }
}

impl harmos_domain::MemorySize for BooleanExpression {
    fn memory_size(&self) -> usize {
        harmos_domain::MemorySize::memory_size(&self.expression)
    }
}

impl ExpressionContext {
    pub fn new(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }

    pub fn with(mut self, path: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.insert(path.into(), value.into());
        self
    }

    pub fn get(&self, path: &str) -> Option<&Value> {
        self.values.get(path)
    }
}

impl Expression {
    pub fn literal(value: impl Into<Value>) -> Self {
        Self {
            kind: ExpressionKind::Literal { value: value.into() },
        }
    }

    pub fn selector(path: impl Into<String>) -> Self {
        Self {
            kind: ExpressionKind::Selector { path: path.into() },
        }
    }

    pub fn unary(op: ExpressionUnaryOp, value: Expression) -> Self {
        Self {
            kind: ExpressionKind::Unary { op, value: Box::new(value) },
        }
    }

    pub fn binary(op: ExpressionBinaryOp, left: Expression, right: Expression) -> Self {
        Self {
            kind: ExpressionKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }

    pub fn call(function: impl Into<String>, args: Vec<Expression>) -> Self {
        Self {
            kind: ExpressionKind::Call { function: function.into(), args },
        }
    }

    pub fn evaluate(&self, context: &ExpressionContext) -> crate::Result<Value> {
        match &self.kind {
            ExpressionKind::Literal { value } => Ok(value.clone()),
            ExpressionKind::Selector { path } => {
                context.get(path).cloned().ok_or_else(|| ExpressionError::UnknownSelector(path.clone()).into())
            }
            ExpressionKind::Unary { op, value } => match op {
                ExpressionUnaryOp::Not => Ok(Value::Bool(!bool_value(&value.evaluate(context)?)?)),
            },
            ExpressionKind::Binary { op, left, right } => {
                let left = left.evaluate(context)?;
                let right = right.evaluate(context)?;
                binary_value(*op, &left, &right).map_err(Into::into)
            }
            ExpressionKind::Call { function, .. } => Err(ExpressionError::UnsupportedCall(function.clone()).into()),
        }
    }

    pub fn evaluate_bool(&self, context: &ExpressionContext) -> crate::Result<bool> {
        bool_value(&self.evaluate(context)?).map_err(Into::into)
    }
}

impl BooleanExpression {
    pub fn new(expression: Expression) -> Self {
        Self { expression }
    }

    pub fn selector(path: impl Into<String>) -> BooleanExpressionBuilder {
        BooleanExpressionBuilder { left: Expression::selector(path) }
    }

    pub fn and(left: BooleanExpression, right: BooleanExpression) -> Self {
        Self::new(Expression::binary(ExpressionBinaryOp::And, left.expression, right.expression))
    }

    pub fn or(left: BooleanExpression, right: BooleanExpression) -> Self {
        Self::new(Expression::binary(ExpressionBinaryOp::Or, left.expression, right.expression))
    }

    pub fn evaluate(&self, context: &ExpressionContext) -> crate::Result<bool> {
        self.expression.evaluate_bool(context)
    }
}

impl BooleanExpressionBuilder {
    pub fn eq(self, value: impl Into<Value>) -> BooleanExpression {
        self.compare(ExpressionBinaryOp::Eq, value)
    }

    pub fn ne(self, value: impl Into<Value>) -> BooleanExpression {
        self.compare(ExpressionBinaryOp::Ne, value)
    }

    pub fn gt(self, value: impl Into<Value>) -> BooleanExpression {
        self.compare(ExpressionBinaryOp::Gt, value)
    }

    pub fn lt(self, value: impl Into<Value>) -> BooleanExpression {
        self.compare(ExpressionBinaryOp::Lt, value)
    }

    fn compare(self, op: ExpressionBinaryOp, value: impl Into<Value>) -> BooleanExpression {
        BooleanExpression::new(Expression::binary(op, self.left, Expression::literal(value)))
    }
}

fn expression_memory(expression: &Expression, depth: usize) -> usize {
    if depth > 64 {
        return usize::MAX;
    }
    let heap = match &expression.kind {
        ExpressionKind::Literal { value } => {
            let owned = harmos_domain::MemorySize::memory_size(value);
            if owned == usize::MAX {
                return usize::MAX;
            }
            owned.saturating_sub(std::mem::size_of_val(value))
        }
        ExpressionKind::Selector { path } => path.capacity(),
        ExpressionKind::Unary { value, .. } => expression_memory(value, depth + 1),
        ExpressionKind::Binary { left, right, .. } => {
            expression_memory(left, depth + 1).saturating_add(expression_memory(right, depth + 1))
        }
        ExpressionKind::Call { function, args } => {
            let mut bytes = function.capacity().saturating_add(args.capacity().saturating_mul(std::mem::size_of::<Expression>()));
            for arg in args {
                let owned = expression_memory(arg, depth + 1);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<Expression>()));
            }
            bytes
        }
    };
    std::mem::size_of::<Expression>().saturating_add(heap)
}

fn binary_value(op: ExpressionBinaryOp, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    Ok(Value::Bool(match op {
        ExpressionBinaryOp::And => bool_value(left)? && bool_value(right)?,
        ExpressionBinaryOp::Or => bool_value(left)? || bool_value(right)?,
        ExpressionBinaryOp::Eq => left == right,
        ExpressionBinaryOp::Ne => left != right,
        ExpressionBinaryOp::Gt => number_order(left, right)?.is_gt(),
        ExpressionBinaryOp::Gte => !number_order(left, right)?.is_lt(),
        ExpressionBinaryOp::Lt => number_order(left, right)?.is_lt(),
        ExpressionBinaryOp::Lte => !number_order(left, right)?.is_gt(),
        ExpressionBinaryOp::Contains => text_value(left)?.contains(text_value(right)?),
        ExpressionBinaryOp::StartsWith => text_value(left)?.starts_with(text_value(right)?),
        ExpressionBinaryOp::EndsWith => text_value(left)?.ends_with(text_value(right)?),
    }))
}

fn bool_value(value: &Value) -> Result<bool, ExpressionError> {
    match value {
        Value::Bool(value) => Ok(*value),
        _ => Err(ExpressionError::ExpectedBoolean),
    }
}

fn number_order(left: &Value, right: &Value) -> Result<std::cmp::Ordering, ExpressionError> {
    numeric_cmp(left, right).ok_or(ExpressionError::ExpectedNumber)
}

fn text_value(value: &Value) -> Result<&str, ExpressionError> {
    match value {
        Value::Text(value) => Ok(value),
        _ => Err(ExpressionError::ExpectedText),
    }
}

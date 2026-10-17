/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod bounds;
pub mod limits;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::collections::BTreeSet;
// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::ParameterId;
use crate::model::values::MeasurementUnit;
use crate::model::values::Value;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */
product_error!(ParameterResolverError, formatter {
    Self::DuplicateParameter(parameter) => write!(formatter, "duplicate parameter definition for {parameter}"),
    Self::UnknownReference { parameter, reference } => write!(formatter, "parameter {parameter:?} references unknown parameter {reference:?}"),
    Self::DependencyCycle(cycle) => write!(formatter, "parameter dependency cycle detected: {cycle}"),
    Self::ExpectedScalar { parameter } => write!(formatter, "parameter {parameter:?} expression requires scalar values"),
    Self::ArrayItemMustBeScalar { parameter } => write!(formatter, "parameter {parameter:?} array items must resolve to scalar values"),
    Self::InvalidRangeStep { parameter } => write!(formatter, "parameter {parameter:?} has invalid range step 0"),
    Self::InvalidLinspaceCount { parameter, count } => write!(formatter, "parameter {parameter:?} has invalid linspace count {count}"),
    Self::IncompatibleOperandSizes { parameter, left, right } => write!(formatter, "parameter {parameter:?} has incompatible operand sizes {left} and {right}"),
    Self::InvalidType { parameter, value, value_type } => write!(formatter, "parameter {parameter:?} produced unsupported value {value:?} for type {value_type:?}"),
    Self::InvalidBounds { parameter } => write!(formatter, "parameter {parameter:?} has invalid numeric bounds"),
    Self::OutOfBounds { parameter, value, min, max } => write!(formatter, "parameter {parameter:?} value {value} is outside inclusive bounds {min:?}..{max:?}"),
    Self::InvalidChoice { parameter, value } => write!(formatter, "parameter {parameter:?} value {value:?} is not an allowed choice"),
    Self::DivisionByZero { parameter } => write!(formatter, "parameter {parameter:?} cannot divide by zero"),
    Self::MissingValue { parameter } => write!(formatter, "parameter {parameter:?} has no value"),
    Self::ExpectedList { parameter, value } => write!(formatter, "list parameter {parameter:?} expects a list of values, got {value:?}"),
    Self::InvalidListItem { parameter, index, value, element_type } => write!(formatter, "list parameter {parameter:?} item {index} is {value:?}, which is not a {element_type:?}"),
    Self::TooFewItems { parameter, count, min_items } => write!(formatter, "list parameter {parameter:?} has {count} items; at least {min_items} are required"),
    Self::TooManyItems { parameter, count, max_items } => write!(formatter, "list parameter {parameter:?} has {count} items; at most {max_items} are allowed"),
});

/* === Type Aliases === */
pub type ParameterGroupId = Identifier;
pub type DynamicDefaultResolver = dyn Fn(DynamicDefaultKey) -> Option<Value> + Send + Sync;

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterDefinition {
    #[serde(default)]
    pub expansion: ParameterExpansion,
    pub parameter_id: ParameterId,
    pub metadata: Metadata,
    #[serde(default)]
    pub group_id: Option<ParameterGroupId>,
    pub value_type: ValueType,
    pub unit: Option<MeasurementUnit>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    pub required: bool,
    pub expression: ParameterExpr,
    #[serde(default)]
    pub runtime_writable: bool,
    #[serde(default)]
    pub selection: Option<ParameterSelection>,
    #[serde(default)]
    pub list: Option<ParameterList>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterList {
    pub element_type: ValueType,
    #[serde(default)]
    pub min_items: Option<usize>,
    #[serde(default)]
    pub max_items: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterGroup {
    pub group_id: ParameterGroupId,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterSelection {
    #[serde(default)]
    pub choices: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterializedParameter {
    pub metadata: Metadata,
    #[serde(default)]
    pub group_id: Option<ParameterGroupId>,
    pub source: ParameterSource,
    pub value: Value,
    pub unit: Option<MeasurementUnit>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedParameterSpace {
    pub definition: ParameterDefinition,
    pub values: Vec<Value>,
}

/* === Enums === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterExpansion {
    #[default]
    Single,
    Cartesian,
    Zip {
        group: Identifier,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterSource {
    Definition,
    Traceability,
    Environment,
    Resource,
    Operator,
    Runtime,
    Telemetry,
    Record,
    Output,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterExpr {
    Literal {
        value: Value,
    },
    Array {
        values: Vec<ParameterExpr>,
    },
    Range {
        start: i64,
        stop: i64,
        #[serde(default = "default_integer_step")]
        step: i64,
    },
    Arange {
        start: f64,
        stop: f64,
        #[serde(default = "default_number_step")]
        step: f64,
    },
    Linspace {
        start: f64,
        stop: f64,
        count: usize,
    },
    Reference {
        parameter: String,
    },
    DynamicDefault {
        key: DynamicDefaultKey,
        #[serde(default)]
        fallback: Option<Box<ParameterExpr>>,
    },
    Unary {
        op: ParameterUnaryOp,
        value: Box<ParameterExpr>,
    },
    Binary {
        op: ParameterBinaryOp,
        left: Box<ParameterExpr>,
        right: Box<ParameterExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicDefaultKey {
    OperatorName,
    OperatorEmail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterUnaryOp {
    Negate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParameterResolverError {
    DuplicateParameter(String),
    UnknownReference { parameter: String, reference: String },
    DependencyCycle(String),
    ExpectedScalar { parameter: String },
    ArrayItemMustBeScalar { parameter: String },
    InvalidRangeStep { parameter: String },
    InvalidLinspaceCount { parameter: String, count: usize },
    IncompatibleOperandSizes { parameter: String, left: usize, right: usize },
    InvalidType { parameter: String, value: Value, value_type: ValueType },
    InvalidBounds { parameter: String },
    OutOfBounds { parameter: String, value: f64, min: Option<f64>, max: Option<f64> },
    InvalidChoice { parameter: String, value: Value },
    DivisionByZero { parameter: String },
    MissingValue { parameter: String },
    ExpectedList { parameter: String, value: Value },
    InvalidListItem { parameter: String, index: usize, value: Value, element_type: ValueType },
    TooFewItems { parameter: String, count: usize, min_items: usize },
    TooManyItems { parameter: String, count: usize, max_items: usize },
}

#[derive(Clone, Debug, PartialEq)]
enum EvaluatedExpr {
    Scalar(Value),
    Space(Vec<Value>),
}

/* === Traits === */

/* === Implementations === */
impl harmos_domain::MemorySize for ParameterExpansion {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Single | Self::Cartesian => 0,
            Self::Zip { group } => {
                let owned = harmos_domain::MemorySize::memory_size(group);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                owned.saturating_sub(std::mem::size_of_val(group))
            }
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos_domain::MemorySize for ParameterList {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl harmos_domain::MemorySize for ParameterSelection {
    fn memory_size(&self) -> usize {
        harmos_domain::MemorySize::memory_size(&self.choices)
    }
}

impl harmos_domain::MemorySize for ParameterGroup {
    fn memory_size(&self) -> usize {
        let metadata = harmos_domain::MemorySize::memory_size(&self.metadata);
        let group = harmos_domain::MemorySize::memory_size(&self.group_id);
        if metadata == usize::MAX || group == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of_val(&self.metadata)))
            .saturating_add(group.saturating_sub(std::mem::size_of_val(&self.group_id)))
    }
}

impl harmos_domain::MemorySize for ParameterDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.expansion.memory_size(), std::mem::size_of_val(&self.expansion)),
            (self.parameter_id.memory_size(), std::mem::size_of_val(&self.parameter_id)),
            (self.metadata.memory_size(), std::mem::size_of_val(&self.metadata)),
            (self.group_id.memory_size(), std::mem::size_of_val(&self.group_id)),
            (self.unit.memory_size(), std::mem::size_of_val(&self.unit)),
            (self.expression.memory_size(), std::mem::size_of_val(&self.expression)),
            (self.selection.memory_size(), std::mem::size_of_val(&self.selection)),
            (self.extensions.memory_size(), std::mem::size_of_val(&self.extensions)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for ParameterExpr {
    fn memory_size(&self) -> usize {
        parameter_expression_memory(self, 0)
    }
}

impl harmos_domain::MemorySize for MaterializedParameter {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.group_id), std::mem::size_of_val(&self.group_id)),
            (harmos_domain::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl ParameterSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Traceability => "traceability",
            Self::Environment => "environment",
            Self::Resource => "resource",
            Self::Operator => "operator",
            Self::Runtime => "runtime",
            Self::Telemetry => "telemetry",
            Self::Record => "record",
            Self::Output => "output",
        }
    }

    pub fn is_derived(&self) -> bool {
        matches!(self, Self::Telemetry | Self::Record | Self::Output)
    }
}

/* === Public Functions === */
pub fn resolve_parameter_spaces(
    definitions: &[ParameterDefinition], dynamic_defaults: &DynamicDefaultResolver,
) -> crate::Result<BTreeMap<String, ResolvedParameterSpace>> {
    resolve_parameter_spaces_detail(definitions, dynamic_defaults).map_err(Into::into)
}

pub(crate) fn resolve_parameter_spaces_detail(
    definitions: &[ParameterDefinition], dynamic_defaults: &DynamicDefaultResolver,
) -> Result<BTreeMap<String, ResolvedParameterSpace>, ParameterResolverError> {
    let mut definitions_by_name = BTreeMap::new();
    for definition in definitions {
        let name = parameter_name(definition).to_owned();
        if definitions_by_name.insert(name.clone(), definition.clone()).is_some() {
            return Err(ParameterResolverError::DuplicateParameter(name));
        }
    }

    let order = topological_order(&definitions_by_name)?;
    let mut evaluated = BTreeMap::<String, EvaluatedExpr>::new();
    let mut spaces = BTreeMap::new();

    for name in order {
        let definition = definitions_by_name.get(&name).expect("topological order only yields known definitions");
        let result = evaluate_expr(&definition.expression, definition, &evaluated, dynamic_defaults)?;
        let values = normalize_space(definition, result)?;
        validate_space(definition, &values)?;
        evaluated.insert(name.clone(), EvaluatedExpr::Space(values.clone()));
        spaces.insert(name, ResolvedParameterSpace { definition: definition.clone(), values });
    }

    Ok(spaces)
}

pub fn materialized_parameter(
    definition: &ParameterDefinition, source: ParameterSource, value: Value,
) -> crate::Result<MaterializedParameter> {
    let value = coerce_definition_value(definition, value)?;
    validate_parameter_value(definition, &value)?;

    Ok(MaterializedParameter {
        metadata: definition.metadata.clone(),
        group_id: definition.group_id.clone(),
        source,
        value,
        unit: definition.unit.clone(),
    })
}

pub fn coerce_definition_value(definition: &ParameterDefinition, value: Value) -> crate::Result<Value> {
    let parameter = parameter_name(definition);
    match &definition.list {
        Some(list) => coerce_list_value_detail(parameter, list, value).map_err(Into::into),
        None => coerce_parameter_value_detail(parameter, &definition.value_type, value).map_err(Into::into),
    }
}

pub fn coerce_list_value(parameter: impl AsRef<str>, list: &ParameterList, value: Value) -> crate::Result<Value> {
    coerce_list_value_detail(parameter, list, value).map_err(Into::into)
}

pub(crate) fn coerce_list_value_detail(
    parameter: impl AsRef<str>, list: &ParameterList, value: Value,
) -> Result<Value, ParameterResolverError> {
    let parameter = parameter.as_ref();

    if matches!(value, Value::Null) {
        return Ok(value);
    }

    let Value::Array(items) = value else {
        return Err(ParameterResolverError::ExpectedList { parameter: parameter.to_owned(), value });
    };

    let items = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            coerce_parameter_value_detail(parameter, &list.element_type, item.clone()).map_err(|_| {
                ParameterResolverError::InvalidListItem {
                    parameter: parameter.to_owned(),
                    index,
                    value: item,
                    element_type: list.element_type.clone(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    validate_list_bounds(parameter, list, items.len())?;

    Ok(Value::Array(items))
}

pub fn coerce_parameter_value(parameter: impl AsRef<str>, value_type: &ValueType, value: Value) -> crate::Result<Value> {
    coerce_parameter_value_detail(parameter, value_type, value).map_err(Into::into)
}

pub(crate) fn coerce_parameter_value_detail(
    parameter: impl AsRef<str>, value_type: &ValueType, value: Value,
) -> Result<Value, ParameterResolverError> {
    let parameter = parameter.as_ref();

    match value_type {
        ValueType::Boolean => match value {
            Value::Bool(_) | Value::Null => Ok(value),
            Value::Integer(value) => Ok(Value::Bool(value != 0)),
            Value::Unsigned(value) => Ok(Value::Bool(value != 0)),
            Value::Number(value) => Ok(Value::Bool(value != 0.0)),
            Value::Text(value) => {
                let normalized = value.trim().to_ascii_lowercase();
                match normalized.as_str() {
                    "" => Ok(Value::Bool(false)),
                    "1" | "true" | "yes" | "on" => Ok(Value::Bool(true)),
                    "0" | "false" | "no" | "off" => Ok(Value::Bool(false)),
                    _ => Err(ParameterResolverError::InvalidType {
                        parameter: parameter.to_owned(),
                        value: Value::Text(value),
                        value_type: value_type.clone(),
                    }),
                }
            }
            value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() }),
        },
        ValueType::Integer => match value {
            Value::Integer(_) | Value::Unsigned(_) | Value::Null => Ok(value),
            Value::Number(value)
                if value.is_finite() && value.fract() == 0.0 && value >= i64::MIN as f64 && value < 9_223_372_036_854_775_808.0 =>
            {
                Ok(Value::Integer(value as i64))
            }
            Value::Text(value) => value.trim().parse::<i64>().map(Value::Integer).map_err(|_| ParameterResolverError::InvalidType {
                parameter: parameter.to_owned(),
                value: Value::Text(value),
                value_type: value_type.clone(),
            }),
            Value::Bool(value) => Ok(Value::Integer(if value { 1 } else { 0 })),
            value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() }),
        },
        ValueType::Number => match value {
            Value::Number(_) | Value::Null => Ok(value),
            Value::Integer(_) | Value::Unsigned(_) => {
                let number = crate::model::values::numeric_value(&value).expect("an integer has a numeric projection");
                if crate::model::values::numeric_cmp(&value, &Value::Number(number)) == Some(std::cmp::Ordering::Equal) {
                    Ok(Value::Number(number))
                } else {
                    Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() })
                }
            }
            Value::Text(value) => value.trim().parse::<f64>().ok().filter(|value| value.is_finite()).map(Value::Number).ok_or_else(|| {
                ParameterResolverError::InvalidType {
                    parameter: parameter.to_owned(),
                    value: Value::Text(value),
                    value_type: value_type.clone(),
                }
            }),
            Value::Bool(value) => Ok(Value::Number(if value { 1.0 } else { 0.0 })),
            value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() }),
        },
        ValueType::Text | ValueType::File => Ok(match value {
            Value::Text(_) | Value::Null => value,
            Value::Bool(value) => Value::Text(value.to_string()),
            Value::Integer(value) => Value::Text(value.to_string()),
            Value::Unsigned(value) => Value::Text(value.to_string()),
            Value::Number(value) => Value::Text(value.to_string()),
            value => value,
        }),
        ValueType::Datetime => match value {
            Value::Datetime(_) | Value::Null => Ok(value),
            value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() }),
        },
        ValueType::Object => match value {
            Value::Object(_) | Value::Null => Ok(value),
            value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: value_type.clone() }),
        },
        ValueType::Array => match value {
            Value::Array(_) | Value::Null => Ok(value),
            value => Ok(Value::Array(vec![value])),
        },
    }
}

pub fn parameter_name(definition: &ParameterDefinition) -> &str {
    definition.metadata.identifier.as_str()
}

pub(crate) fn validate_parameter_value(definition: &ParameterDefinition, value: &Value) -> Result<(), ParameterResolverError> {
    validate_numeric_bounds(definition, value)?;
    if matches!(value, Value::Null) {
        return if definition.required {
            Err(ParameterResolverError::MissingValue { parameter: parameter_name(definition).to_owned() })
        } else {
            Ok(())
        };
    }
    let Some(selection) = &definition.selection else {
        return Ok(());
    };
    if selection.choices.is_empty() {
        return Ok(());
    }

    let name = parameter_name(definition).to_owned();
    // A list's choices are its elements' choices, exactly as a multi-select's
    // are; both carry their selection in one `Value::Array`.
    let element_wise = definition.list.is_some();
    let values = match value {
        Value::Array(items) if element_wise => items.as_slice(),
        value => std::slice::from_ref(value),
    };

    for value in values {
        if !selection.choices.iter().any(|choice| choice == value) {
            return Err(ParameterResolverError::InvalidChoice { parameter: name.clone(), value: value.clone() });
        }
    }

    Ok(())
}

/* === Private Functions === */
fn parameter_expression_memory(expression: &ParameterExpr, depth: usize) -> usize {
    if depth > 64 {
        return usize::MAX;
    }
    let heap = match expression {
        ParameterExpr::Literal { value } => {
            let bytes = harmos_domain::MemorySize::memory_size(value);
            if bytes == usize::MAX {
                return usize::MAX;
            }
            bytes.saturating_sub(std::mem::size_of_val(value))
        }
        ParameterExpr::Reference { parameter } => parameter.capacity(),
        ParameterExpr::Array { values } => {
            let mut bytes = values.capacity().saturating_mul(std::mem::size_of::<ParameterExpr>());
            for value in values {
                let owned = parameter_expression_memory(value, depth + 1);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<ParameterExpr>()));
            }
            bytes
        }
        ParameterExpr::DynamicDefault { fallback, .. } => {
            fallback.as_deref().map_or(0, |value| parameter_expression_memory(value, depth + 1))
        }
        ParameterExpr::Unary { value, .. } => parameter_expression_memory(value, depth + 1),
        ParameterExpr::Binary { left, right, .. } => {
            parameter_expression_memory(left, depth + 1).saturating_add(parameter_expression_memory(right, depth + 1))
        }
        ParameterExpr::Range { .. } | ParameterExpr::Arange { .. } | ParameterExpr::Linspace { .. } => 0,
    };
    std::mem::size_of::<ParameterExpr>().saturating_add(heap)
}

fn default_integer_step() -> i64 {
    1
}

fn default_number_step() -> f64 {
    1.0
}

fn topological_order(definitions: &BTreeMap<String, ParameterDefinition>) -> Result<Vec<String>, ParameterResolverError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn visit(
        name: &str, definitions: &BTreeMap<String, ParameterDefinition>, states: &mut BTreeMap<String, VisitState>,
        order: &mut Vec<String>, stack: &mut Vec<String>,
    ) -> Result<(), ParameterResolverError> {
        if states.get(name) == Some(&VisitState::Visited) {
            return Ok(());
        }
        if states.get(name) == Some(&VisitState::Visiting) {
            stack.push(name.to_owned());
            return Err(ParameterResolverError::DependencyCycle(stack.join(" -> ")));
        }

        states.insert(name.to_owned(), VisitState::Visiting);
        stack.push(name.to_owned());

        let definition = definitions.get(name).expect("visited dependency must exist in definitions");
        for dependency in expr_dependencies(&definition.expression) {
            if !definitions.contains_key(&dependency) {
                return Err(ParameterResolverError::UnknownReference { parameter: name.to_owned(), reference: dependency });
            }
            visit(&dependency, definitions, states, order, stack)?;
        }

        stack.pop();
        states.insert(name.to_owned(), VisitState::Visited);
        order.push(name.to_owned());
        Ok(())
    }

    let mut states = BTreeMap::new();
    let mut order = Vec::with_capacity(definitions.len());
    for name in definitions.keys() {
        if !states.contains_key(name) {
            visit(name, definitions, &mut states, &mut order, &mut Vec::new())?;
        }
    }

    Ok(order)
}

fn expr_dependencies(expr: &ParameterExpr) -> BTreeSet<String> {
    let mut dependencies = BTreeSet::new();
    collect_expr_dependencies(expr, &mut dependencies);
    dependencies
}

fn collect_expr_dependencies(expr: &ParameterExpr, dependencies: &mut BTreeSet<String>) {
    match expr {
        ParameterExpr::Literal { .. }
        | ParameterExpr::Range { .. }
        | ParameterExpr::Arange { .. }
        | ParameterExpr::Linspace { .. }
        | ParameterExpr::DynamicDefault { fallback: None, .. } => {}
        ParameterExpr::Array { values } => {
            for value in values {
                collect_expr_dependencies(value, dependencies);
            }
        }
        ParameterExpr::Reference { parameter } => {
            dependencies.insert(parameter.clone());
        }
        ParameterExpr::DynamicDefault { fallback: Some(fallback), .. } | ParameterExpr::Unary { value: fallback, .. } => {
            collect_expr_dependencies(fallback, dependencies);
        }
        ParameterExpr::Binary { left, right, .. } => {
            collect_expr_dependencies(left, dependencies);
            collect_expr_dependencies(right, dependencies);
        }
    }
}

fn evaluate_expr(
    expr: &ParameterExpr, definition: &ParameterDefinition, evaluated: &BTreeMap<String, EvaluatedExpr>,
    dynamic_defaults: &DynamicDefaultResolver,
) -> Result<EvaluatedExpr, ParameterResolverError> {
    let name = parameter_name(definition).to_owned();

    match expr {
        ParameterExpr::Literal { value } => Ok(EvaluatedExpr::Scalar(value.clone())),
        ParameterExpr::Array { values } => values
            .iter()
            .map(|value| match evaluate_expr(value, definition, evaluated, dynamic_defaults)? {
                EvaluatedExpr::Scalar(value) => Ok(value),
                EvaluatedExpr::Space(_) => Err(ParameterResolverError::ArrayItemMustBeScalar { parameter: name.clone() }),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(EvaluatedExpr::Space),
        ParameterExpr::Range { start, stop, step } => {
            if *step == 0 {
                return Err(ParameterResolverError::InvalidRangeStep { parameter: name });
            }

            let values = if *step > 0 {
                (*start..=*stop).step_by(*step as usize).map(Value::Integer).collect()
            } else {
                let mut current = *start;
                let mut values = Vec::new();
                while current >= *stop {
                    values.push(Value::Integer(current));
                    current += *step;
                }
                values
            };
            Ok(EvaluatedExpr::Space(values))
        }
        ParameterExpr::Arange { start, stop, step } => {
            if *step == 0.0 {
                return Err(ParameterResolverError::InvalidRangeStep { parameter: name });
            }

            let mut current = *start;
            let mut values = Vec::new();
            if *step > 0.0 {
                while current < *stop {
                    values.push(Value::Number(round_number(current)));
                    current += *step;
                }
            } else {
                while current > *stop {
                    values.push(Value::Number(round_number(current)));
                    current += *step;
                }
            }
            Ok(EvaluatedExpr::Space(values))
        }
        ParameterExpr::Linspace { start, stop, count } => {
            if *count == 0 {
                return Err(ParameterResolverError::InvalidLinspaceCount { parameter: name, count: *count });
            }
            if *count == 1 {
                return Ok(EvaluatedExpr::Space(vec![Value::Number(round_number(*start))]));
            }

            let step = (*stop - *start) / (*count as f64 - 1.0);
            Ok(EvaluatedExpr::Space((0..*count).map(|index| Value::Number(round_number(*start + step * index as f64))).collect()))
        }
        ParameterExpr::Reference { parameter } => evaluated
            .get(parameter)
            .cloned()
            .ok_or_else(|| ParameterResolverError::UnknownReference { parameter: name, reference: parameter.clone() }),
        ParameterExpr::DynamicDefault { key, fallback } => {
            if let Some(value) = dynamic_defaults(key.clone()) {
                return Ok(EvaluatedExpr::Scalar(value));
            }
            let Some(fallback) = fallback else {
                return Ok(EvaluatedExpr::Scalar(Value::Null));
            };
            evaluate_expr(fallback, definition, evaluated, dynamic_defaults)
        }
        ParameterExpr::Unary { op, value } => {
            let value = evaluate_expr(value, definition, evaluated, dynamic_defaults)?;
            map_unary(name, op, value)
        }
        ParameterExpr::Binary { op, left, right } => {
            let left = evaluate_expr(left, definition, evaluated, dynamic_defaults)?;
            let right = evaluate_expr(right, definition, evaluated, dynamic_defaults)?;
            map_binary(name, op, left, right)
        }
    }
}

fn map_unary(parameter: String, op: &ParameterUnaryOp, value: EvaluatedExpr) -> Result<EvaluatedExpr, ParameterResolverError> {
    match value {
        EvaluatedExpr::Scalar(value) => Ok(EvaluatedExpr::Scalar(apply_unary(&parameter, op, value)?)),
        EvaluatedExpr::Space(values) => {
            values.into_iter().map(|value| apply_unary(&parameter, op, value)).collect::<Result<Vec<_>, _>>().map(EvaluatedExpr::Space)
        }
    }
}

fn map_binary(
    parameter: String, op: &ParameterBinaryOp, left: EvaluatedExpr, right: EvaluatedExpr,
) -> Result<EvaluatedExpr, ParameterResolverError> {
    match (left, right) {
        (EvaluatedExpr::Scalar(left), EvaluatedExpr::Scalar(right)) => {
            Ok(EvaluatedExpr::Scalar(apply_binary(&parameter, op, left, right)?))
        }
        (EvaluatedExpr::Scalar(left), EvaluatedExpr::Space(rights)) => rights
            .into_iter()
            .map(|right| apply_binary(&parameter, op, left.clone(), right))
            .collect::<Result<Vec<_>, _>>()
            .map(EvaluatedExpr::Space),
        (EvaluatedExpr::Space(lefts), EvaluatedExpr::Scalar(right)) => lefts
            .into_iter()
            .map(|left| apply_binary(&parameter, op, left, right.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map(EvaluatedExpr::Space),
        (EvaluatedExpr::Space(lefts), EvaluatedExpr::Space(rights)) => {
            if lefts.len() == rights.len() {
                return lefts
                    .into_iter()
                    .zip(rights)
                    .map(|(left, right)| apply_binary(&parameter, op, left, right))
                    .collect::<Result<Vec<_>, _>>()
                    .map(EvaluatedExpr::Space);
            }
            if lefts.len() == 1 {
                let left = lefts.into_iter().next().ok_or_else(|| ParameterResolverError::MissingValue { parameter: parameter.clone() })?;
                return rights
                    .into_iter()
                    .map(|right| apply_binary(&parameter, op, left.clone(), right))
                    .collect::<Result<Vec<_>, _>>()
                    .map(EvaluatedExpr::Space);
            }
            if rights.len() == 1 {
                let right =
                    rights.into_iter().next().ok_or_else(|| ParameterResolverError::MissingValue { parameter: parameter.clone() })?;
                return lefts
                    .into_iter()
                    .map(|left| apply_binary(&parameter, op, left, right.clone()))
                    .collect::<Result<Vec<_>, _>>()
                    .map(EvaluatedExpr::Space);
            }

            Err(ParameterResolverError::IncompatibleOperandSizes { parameter, left: lefts.len(), right: rights.len() })
        }
    }
}

fn apply_unary(parameter: &str, op: &ParameterUnaryOp, value: Value) -> Result<Value, ParameterResolverError> {
    let numeric = numeric_value(parameter, value)?;

    match op {
        ParameterUnaryOp::Negate => Ok(Value::Number(-numeric)),
    }
}

fn apply_binary(parameter: &str, op: &ParameterBinaryOp, left: Value, right: Value) -> Result<Value, ParameterResolverError> {
    let left = numeric_value(parameter, left)?;
    let right = numeric_value(parameter, right)?;

    let value = match op {
        ParameterBinaryOp::Add => left + right,
        ParameterBinaryOp::Subtract => left - right,
        ParameterBinaryOp::Multiply => left * right,
        ParameterBinaryOp::Divide => {
            if right == 0.0 {
                return Err(ParameterResolverError::DivisionByZero { parameter: parameter.to_owned() });
            }
            left / right
        }
    };

    Ok(Value::Number(round_number(value)))
}

fn numeric_value(parameter: &str, value: Value) -> Result<f64, ParameterResolverError> {
    match value {
        Value::Number(value) => Ok(value),
        Value::Integer(value) => Ok(value as f64),
        value => Err(ParameterResolverError::InvalidType { parameter: parameter.to_owned(), value, value_type: ValueType::Number }),
    }
}

fn normalize_space(definition: &ParameterDefinition, result: EvaluatedExpr) -> Result<Vec<Value>, ParameterResolverError> {
    let name = parameter_name(definition);

    let values = match result {
        EvaluatedExpr::Scalar(value) => vec![value],
        EvaluatedExpr::Space(values) => values,
    };

    values
        .into_iter()
        .map(|value| {
            if let Some(list) = &definition.list {
                coerce_list_value_detail(name, list, value)
            } else {
                coerce_parameter_value_detail(name, &definition.value_type, value)
            }
        })
        .collect()
}

fn validate_list_bounds(parameter: &str, list: &ParameterList, count: usize) -> Result<(), ParameterResolverError> {
    if let Some(min_items) = list.min_items
        && count < min_items
    {
        return Err(ParameterResolverError::TooFewItems { parameter: parameter.to_owned(), count, min_items });
    }
    if let Some(max_items) = list.max_items
        && count > max_items
    {
        return Err(ParameterResolverError::TooManyItems { parameter: parameter.to_owned(), count, max_items });
    }

    Ok(())
}

fn validate_space(definition: &ParameterDefinition, values: &[Value]) -> Result<(), ParameterResolverError> {
    if values.is_empty() && definition.required {
        return Err(ParameterResolverError::MissingValue { parameter: parameter_name(definition).to_owned() });
    }

    for value in values {
        validate_parameter_value(definition, value)?;
    }

    Ok(())
}

fn validate_numeric_bounds(definition: &ParameterDefinition, value: &Value) -> Result<(), ParameterResolverError> {
    if definition.min.is_none() && definition.max.is_none() {
        return Ok(());
    }
    let name = parameter_name(definition).to_owned();
    let numeric_type = definition.list.as_ref().map_or(&definition.value_type, |list| &list.element_type);
    if !matches!(numeric_type, ValueType::Integer | ValueType::Number)
        || definition.min.is_some_and(|bound| !bound.is_finite())
        || definition.max.is_some_and(|bound| !bound.is_finite())
        || matches!((definition.min, definition.max), (Some(min), Some(max)) if min > max)
    {
        return Err(ParameterResolverError::InvalidBounds { parameter: name });
    }
    let values = match value {
        Value::Array(items) if definition.list.is_some() => items.as_slice(),
        value => std::slice::from_ref(value),
    };
    for value in values {
        let number = match value {
            Value::Integer(value) => *value as f64,
            Value::Number(value) => *value,
            _ => continue,
        };
        if !number.is_finite() || definition.min.is_some_and(|min| number < min) || definition.max.is_some_and(|max| number > max) {
            return Err(ParameterResolverError::OutOfBounds { parameter: name, value: number, min: definition.min, max: definition.max });
        }
    }
    Ok(())
}

fn round_number(value: f64) -> f64 {
    (value * 1_000_000_000_000.0).round() / 1_000_000_000_000.0
}

/* === Tests === */

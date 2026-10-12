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
use super::{Map, Value};
use crate::core::Identifier;
use crate::definition::RuntimeCondition;
use crate::records::RecordCondition;
use crate::storage::{Error, Result};
use crate::telemetry::{TelemetryCondition, TelemetryConditionOperator};

/* === Constants === */
const MAX_NODES: usize = 4096;
const MAX_DEPTH: usize = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct Condition {
    nodes: Vec<Node>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Node {
    kind: String,
    children: u32,
    record_type: Option<String>,
    match_fields: Option<Map>,
    stream: Option<String>,
    field: Option<String>,
    operator: Option<String>,
    target: Option<Value>,
    tolerance: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Condition {
    pub(super) fn encoding_memory(value: &RuntimeCondition) -> Result<usize> {
        let mut count = 0;
        let memory = encoding_memory(value, 0, &mut count)?;
        Ok(std::mem::size_of::<Self>().saturating_add(memory))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        if self.nodes.is_empty() || self.nodes.len() > MAX_NODES {
            return Err(Error::Invalid("condition exceeds node limit or is empty"));
        }
        let mut at = 0;
        let bound = condition_memory(&self.nodes, &mut at, 0)?;
        if at != self.nodes.len() {
            return Err(Error::Invalid("condition has trailing nodes"));
        }
        Ok(bound)
    }
}

impl TryFrom<&RuntimeCondition> for Condition {
    type Error = Error;

    fn try_from(value: &RuntimeCondition) -> Result<Self> {
        let mut count = 0;
        let memory = encoding_memory(value, 0, &mut count)?;
        if std::mem::size_of::<Self>().saturating_add(memory) == usize::MAX {
            return Err(Error::Invalid("condition encoding memory overflows"));
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(count)
            .map_err(|_| Error::Invalid("condition nodes allocation failed"))?;
        append_condition(value, &mut nodes)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Condition> for RuntimeCondition {
    type Error = Error;

    fn try_from(row: Condition) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("condition projection memory overflows"));
        }
        let mut nodes = row.nodes.into_iter();
        consume_condition(&mut nodes, 0)
    }
}

impl Serialize for Condition {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let value: RuntimeCondition = self.clone().try_into().map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let source = crate::definition::RuntimeCondition::deserialize(deserializer)?;
        Self::try_from(&source).map_err(serde::de::Error::custom)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn encoding_memory(value: &RuntimeCondition, depth: usize, count: &mut usize) -> Result<usize> {
    if depth > MAX_DEPTH || *count >= MAX_NODES {
        return Err(Error::Invalid("condition exceeds node or depth limit"));
    }
    *count += 1;
    let mut bytes = std::mem::size_of::<Node>();
    match value {
        RuntimeCondition::Record { condition } => {
            let fields = Map::encoding_memory(&condition.match_fields)?;
            if fields == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add("record".len())
                .saturating_add(condition.record_type.as_str().len())
                .saturating_add(fields.saturating_sub(std::mem::size_of::<Map>()));
        }
        RuntimeCondition::Telemetry { condition } => {
            if condition.tolerance.is_some_and(|value| !value.is_finite()) {
                return Err(Error::Invalid("condition tolerance is nonfinite"));
            }
            let target = condition
                .target
                .as_ref()
                .map(Value::encoding_memory)
                .transpose()?
                .unwrap_or(0);
            if target == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add("telemetry".len())
                .saturating_add(condition.stream.as_str().len())
                .saturating_add(condition.field.len())
                .saturating_add(operator_name(&condition.operator).len())
                .saturating_add(target.saturating_sub(std::mem::size_of::<Value>()));
        }
        RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } => {
            u32::try_from(conditions.len()).map_err(|_| Error::Invalid("too many conditions"))?;
            bytes = bytes.saturating_add(3);
            for condition in conditions {
                bytes = bytes.saturating_add(encoding_memory(condition, depth + 1, count)?);
            }
        }
    }
    Ok(bytes)
}

fn append_condition(value: &RuntimeCondition, nodes: &mut Vec<Node>) -> Result<()> {
    let mut node = Node::default();
    let kind = match value {
        RuntimeCondition::Record { condition } => {
            node.record_type = Some(condition.record_type.as_str().to_owned());
            node.match_fields = Some(Map::try_from(&condition.match_fields)?);
            "record"
        }
        RuntimeCondition::Telemetry { condition } => {
            node.stream = Some(condition.stream.as_str().to_owned());
            node.field = Some(condition.field.as_str().to_owned());
            node.operator = Some(operator_name(&condition.operator).to_owned());
            node.target = condition.target.as_ref().map(Value::try_from).transpose()?;
            node.tolerance = condition.tolerance;
            "telemetry"
        }
        RuntimeCondition::All { conditions } => {
            node.children = conditions.len() as u32;
            "all"
        }
        RuntimeCondition::Any { conditions } => {
            node.children = conditions.len() as u32;
            "any"
        }
    };
    node.kind = kind.to_owned();
    nodes.push(node);
    if let RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } = value {
        for condition in conditions {
            append_condition(condition, nodes)?;
        }
    }
    Ok(())
}

fn operator_name(operator: &TelemetryConditionOperator) -> &'static str {
    match operator {
        TelemetryConditionOperator::Exists => "exists",
        TelemetryConditionOperator::Eq => "eq",
        TelemetryConditionOperator::Gt => "gt",
        TelemetryConditionOperator::Gte => "gte",
        TelemetryConditionOperator::Lt => "lt",
        TelemetryConditionOperator::Lte => "lte",
        TelemetryConditionOperator::Within => "within",
    }
}

fn validate_node(node: &Node) -> Result<()> {
    let kind = node.kind.as_str();
    let arity = match kind {
        "record" | "telemetry" => node.children == 0,
        "all" | "any" => true,
        _ => false,
    };
    let required = [
        (node.record_type.is_some(), kind == "record"),
        (node.match_fields.is_some(), kind == "record"),
        (node.stream.is_some(), kind == "telemetry"),
        (node.field.is_some(), kind == "telemetry"),
        (node.operator.is_some(), kind == "telemetry"),
    ];
    if !arity
        || required
            .iter()
            .any(|(present, expected)| present != expected)
        || (kind != "telemetry" && (node.target.is_some() || node.tolerance.is_some()))
    {
        return Err(Error::Invalid("condition fields or arity differ"));
    }
    if node.tolerance.is_some_and(|value| !value.is_finite()) {
        return Err(Error::Invalid("condition tolerance is nonfinite"));
    }
    if let Some(operator) = &node.operator {
        telemetry_operator(operator)?;
    }
    Ok(())
}

fn condition_memory(nodes: &[Node], at: &mut usize, depth: usize) -> Result<usize> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("condition exceeds depth limit"));
    }
    let node = nodes
        .get(*at)
        .ok_or(Error::Invalid("condition child is missing"))?;
    *at += 1;
    validate_node(node)?;
    if node.children as usize > nodes.len().saturating_sub(*at) {
        return Err(Error::Invalid("condition children exceed remaining nodes"));
    }
    let fields = [
        node.record_type.as_ref().map_or(0, String::capacity),
        node.stream.as_ref().map_or(0, String::capacity),
        node.field.as_ref().map_or(0, String::capacity),
        node.match_fields
            .as_ref()
            .map(Map::projection_memory)
            .transpose()?
            .unwrap_or(0),
        node.target
            .as_ref()
            .map(Value::projection_memory)
            .transpose()?
            .unwrap_or(0),
    ];
    let mut bytes = fields.into_iter().fold(
        std::mem::size_of::<RuntimeCondition>(),
        usize::saturating_add,
    );
    for _ in 0..node.children {
        bytes = bytes.saturating_add(condition_memory(nodes, at, depth + 1)?);
    }
    Ok(bytes)
}

fn consume_condition(
    nodes: &mut std::vec::IntoIter<Node>,
    depth: usize,
) -> Result<RuntimeCondition> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("condition exceeds depth limit"));
    }
    let node = nodes
        .next()
        .ok_or(Error::Invalid("condition child is missing"))?;
    match node.kind.as_str() {
        "record" => Ok(RuntimeCondition::Record {
            condition: RecordCondition {
                record_type: Identifier::new(node.record_type.unwrap())?,
                match_fields: node.match_fields.unwrap().try_into()?,
            },
        }),
        "telemetry" => Ok(RuntimeCondition::Telemetry {
            condition: TelemetryCondition {
                stream: Identifier::new(node.stream.unwrap())?,
                field: node.field.unwrap(),
                operator: telemetry_operator(node.operator.as_deref().unwrap())?,
                target: node.target.map(TryInto::try_into).transpose()?,
                tolerance: node.tolerance,
            },
        }),
        "all" | "any" => {
            let mut conditions = Vec::new();
            conditions
                .try_reserve_exact(node.children as usize)
                .map_err(|_| Error::Invalid("condition children allocation failed"))?;
            for _ in 0..node.children {
                conditions.push(consume_condition(nodes, depth + 1)?);
            }
            if node.kind == "all" {
                Ok(RuntimeCondition::All { conditions })
            } else {
                Ok(RuntimeCondition::Any { conditions })
            }
        }
        _ => Err(Error::Invalid("unknown condition kind")),
    }
}

fn telemetry_operator(operator: &str) -> Result<TelemetryConditionOperator> {
    match operator {
        "exists" => Ok(TelemetryConditionOperator::Exists),
        "eq" => Ok(TelemetryConditionOperator::Eq),
        "gt" => Ok(TelemetryConditionOperator::Gt),
        "gte" => Ok(TelemetryConditionOperator::Gte),
        "lt" => Ok(TelemetryConditionOperator::Lt),
        "lte" => Ok(TelemetryConditionOperator::Lte),
        "within" => Ok(TelemetryConditionOperator::Within),
        _ => Err(Error::Invalid("invalid telemetry condition operator")),
    }
}

/* === Tests === */

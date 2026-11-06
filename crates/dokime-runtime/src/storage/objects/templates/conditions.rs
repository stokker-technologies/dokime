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
use crate::definition::RuntimeCondition;
use crate::records::RecordCondition;
use crate::storage::{Error, Result};
use crate::telemetry::{TelemetryCondition, TelemetryConditionOperator};
use dokime_domain::model::identity::Identifier;

/* === Constants === */
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
impl TryFrom<&RuntimeCondition> for Condition {
    type Error = Error;

    fn try_from(value: &RuntimeCondition) -> Result<Self> {
        let mut nodes = Vec::new();
        append_condition(value, &mut nodes, 0)?;
        Ok(Self { nodes })
    }
}

impl TryFrom<Condition> for RuntimeCondition {
    type Error = Error;

    fn try_from(row: Condition) -> Result<Self> {
        let mut nodes = row.nodes.into_iter();
        let condition = consume_condition(&mut nodes, 0)?;
        if nodes.next().is_some() {
            return Err(Error::Invalid("condition has trailing nodes"));
        }
        Ok(condition)
    }
}

impl Serialize for Condition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let value: RuntimeCondition = self.clone().try_into().map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let source = crate::definition::RuntimeCondition::deserialize(deserializer)?;
        Self::try_from(&source).map_err(serde::de::Error::custom)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn append_condition(value: &RuntimeCondition, nodes: &mut Vec<Node>, depth: usize) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("condition exceeds depth limit"));
    }
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
            if condition.tolerance.is_some_and(|value| !value.is_finite()) {
                return Err(Error::Invalid("condition tolerance is nonfinite"));
            }
            node.tolerance = condition.tolerance;
            "telemetry"
        }
        RuntimeCondition::All { conditions } => {
            node.children = u32::try_from(conditions.len()).map_err(|_| Error::Invalid("too many conditions"))?;
            "all"
        }
        RuntimeCondition::Any { conditions } => {
            node.children = u32::try_from(conditions.len()).map_err(|_| Error::Invalid("too many conditions"))?;
            "any"
        }
    };
    node.kind = kind.to_owned();
    nodes.push(node);
    if let RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } = value {
        for condition in conditions {
            append_condition(condition, nodes, depth + 1)?;
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
        || required.iter().any(|(present, expected)| present != expected)
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

fn consume_condition(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<RuntimeCondition> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("condition exceeds depth limit"));
    }
    let node = nodes.next().ok_or(Error::Invalid("condition child is missing"))?;
    validate_node(&node)?;
    if node.children as usize > nodes.len() {
        return Err(Error::Invalid("condition children exceed remaining nodes"));
    }
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
            conditions.try_reserve_exact(node.children as usize).map_err(|_| Error::Invalid("condition children allocation failed"))?;
            for _ in 0..node.children {
                conditions.push(consume_condition(nodes, depth + 1)?);
            }
            if node.kind == "all" { Ok(RuntimeCondition::All { conditions }) } else { Ok(RuntimeCondition::Any { conditions }) }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf() -> RuntimeCondition {
        RuntimeCondition::Telemetry {
            condition: TelemetryCondition {
                stream: Identifier::new("stream").unwrap(),
                field: "value".into(),
                operator: TelemetryConditionOperator::Exists,
                target: None,
                tolerance: None,
            },
        }
    }

    #[test]
    fn admits_a_valid_condition_tree_above_the_former_node_limit() {
        let conditions = (0..4096).map(|_| leaf()).collect();
        let source = RuntimeCondition::All { conditions };
        let row = Condition::try_from(&source).unwrap();
        assert_eq!(row.nodes.len(), 4097);
        let decoded = RuntimeCondition::try_from(row).unwrap();
        assert_eq!(decoded, source);
    }

    #[test]
    fn rejects_trees_past_the_depth_limit_and_trailing_nodes() {
        let nested = |depth| (0..depth).fold(leaf(), |inner, _| RuntimeCondition::All { conditions: vec![inner] });
        assert!(Condition::try_from(&nested(MAX_DEPTH)).is_ok());
        assert!(Condition::try_from(&nested(MAX_DEPTH + 1)).is_err());

        let mut row = Condition::try_from(&nested(MAX_DEPTH)).unwrap();
        row.nodes.insert(
            0,
            Node {
                kind: "all".into(),
                children: 1,
                ..Default::default()
            },
        );
        assert!(RuntimeCondition::try_from(row).is_err());

        let mut trailing = Condition::try_from(&leaf()).unwrap();
        trailing.nodes.push(trailing.nodes[0].clone());
        assert!(RuntimeCondition::try_from(trailing).is_err());
    }
}

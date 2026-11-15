/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompt Validation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{cmp::Ordering, collections::HashMap};

// Third Party Imports

// Local Imports
use crate::{Error, Result};
use dokime_protocol::model::AnswerOperator;
use dokime_protocol::model::AnswerPredicate;
use dokime_protocol::model::EvidenceCondition;
use dokime_protocol::model::Prompt;
use dokime_protocol::model::RecordContracts;
use dokime_protocol::model::StreamMatchOperator;
use dokime_protocol::model::Value;
use dokime_protocol::model::ValueType;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_storage::ExecutionSnapshot;
use harmos_protocol::model::Declarations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn validate(
    definition: &Prompt, action: &str, accepted: bool, values: &HashMap<String, Value>, snapshot: &ExecutionSnapshot,
) -> Result<()> {
    let action = definition
        .actions
        .iter()
        .find(|candidate| candidate.identifier == action)
        .ok_or_else(|| Error::Invalid("Answer Prompt: Choose an Action Declared by the Question".into()))?;
    if action.requires_condition
        && !definition.condition.as_ref().map(|condition| condition_met(condition, snapshot)).transpose()?.unwrap_or(false)
    {
        return Err(Error::Precondition("Answer Prompt: Wait Until the Required Evidence Condition Is Satisfied".into()));
    }
    if !accepted {
        return Ok(());
    }
    for key in values.keys() {
        if !definition.fields.iter().any(|field| field.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == key)) {
            return Err(Error::Invalid(format!("Answer Prompt: Remove the Unknown Field {key}")));
        }
    }
    for field in &definition.fields {
        let name = &field
            .metadata
            .as_ref()
            .ok_or_else(|| Error::Invalid("Answer Prompt: Reconnect the Plugin with Named Fields".into()))?
            .identifier;
        let Some(value) = values.get(name) else {
            if field.required {
                return Err(Error::Invalid(format!("Answer Prompt: Supply the Required Field {name}")));
            }
            continue;
        };
        let items = if field.multiple {
            let Some(ValueKind::List(list)) = &value.kind else {
                return Err(Error::Invalid(format!("Answer Prompt: Supply a List for {name}")));
            };
            list.values.as_slice()
        } else {
            std::slice::from_ref(value)
        };
        if let Some(validation) = &field.validation
            && field.multiple
            && (validation.min_items.is_some_and(|min| items.len() < min as usize)
                || validation.max_items.is_some_and(|max| items.len() > max as usize))
        {
            return Err(Error::Invalid(format!("Answer Prompt: Use the Declared Item Count for {name}")));
        }
        for value in items {
            if !type_matches(field.r#type, value) || (!field.choices.is_empty() && !field.choices.contains(value)) {
                return Err(Error::Invalid(format!("Answer Prompt: Use the Declared Type and Choices for {name}")));
            }
            let Some(validation) = &field.validation else {
                continue;
            };
            if let Some(ValueKind::Text(text)) = &value.kind {
                let length = text.chars().count() as u64;
                let pattern_matches = validation
                    .pattern
                    .as_ref()
                    .map(|pattern| regex::Regex::new(pattern).map(|pattern| pattern.is_match(text)))
                    .transpose()
                    .map_err(|error| Error::Invalid(format!("Answer Prompt: Repair the Field Pattern: {error}")))?
                    .unwrap_or(true);
                if validation.min_length.is_some_and(|min| length < min)
                    || validation.max_length.is_some_and(|max| length > max)
                    || !pattern_matches
                {
                    return Err(Error::Invalid(format!("Answer Prompt: Match the Declared Text Format for {name}")));
                }
            }
            if validation
                .minimum
                .as_ref()
                .is_some_and(|minimum| !matches!(compare(value, minimum), Some(Ordering::Equal | Ordering::Greater)))
                || validation
                    .maximum
                    .as_ref()
                    .is_some_and(|maximum| !matches!(compare(value, maximum), Some(Ordering::Equal | Ordering::Less)))
            {
                return Err(Error::Invalid(format!("Answer Prompt: Use the Declared Range for {name}")));
            }
        }
    }
    for rule in &definition.rules {
        let predicate =
            rule.predicate.as_ref().ok_or_else(|| Error::Invalid("Answer Prompt: Repair the Missing Answer Predicate".into()))?;
        if evaluate(predicate, values)?.kind != Some(ValueKind::Boolean(true)) {
            return Err(Error::Invalid(format!("Answer Prompt: {}", rule.message)));
        }
    }
    Ok(())
}

/* === Private Functions === */

pub(crate) fn type_matches(kind: i32, value: &Value) -> bool {
    use dokime_protocol::model::value::Kind;
    matches!(
        (ValueType::try_from(kind), &value.kind),
        (Ok(ValueType::Boolean), Some(Kind::Boolean(_)))
            | (Ok(ValueType::Integer), Some(Kind::Integer(_)))
            | (Ok(ValueType::UnsignedInteger), Some(Kind::UnsignedInteger(_)))
            | (Ok(ValueType::Number), Some(Kind::Number(_)))
            | (Ok(ValueType::Text), Some(Kind::Text(_)))
            | (Ok(ValueType::Bytes), Some(Kind::Data(_)))
            | (Ok(ValueType::List), Some(Kind::List(_)))
            | (Ok(ValueType::Datetime), Some(Kind::Datetime(_)))
    ) && !matches!(value.kind, Some(Kind::Number(number)) if !number.is_finite())
}

fn compare(left: &Value, right: &Value) -> Option<Ordering> {
    use dokime_protocol::model::value::Kind;
    match (&left.kind, &right.kind) {
        (Some(Kind::Number(left)), Some(Kind::Number(right))) => left.partial_cmp(right),
        (Some(Kind::Integer(left)), Some(Kind::Integer(right))) => Some(left.cmp(right)),
        (Some(Kind::UnsignedInteger(left)), Some(Kind::UnsignedInteger(right))) => Some(left.cmp(right)),
        (Some(Kind::Integer(left)), Some(Kind::UnsignedInteger(right))) => {
            Some(if *left < 0 { Ordering::Less } else { (*left as u64).cmp(right) })
        }
        (Some(Kind::UnsignedInteger(left)), Some(Kind::Integer(right))) => {
            Some(if *right < 0 { Ordering::Greater } else { left.cmp(&(*right as u64)) })
        }
        (Some(Kind::Text(left)), Some(Kind::Text(right))) => Some(left.cmp(right)),
        (Some(Kind::Boolean(left)), Some(Kind::Boolean(right))) => Some(left.cmp(right)),
        (Some(Kind::Datetime(left)), Some(Kind::Datetime(right))) => Some((left.seconds, left.nanos).cmp(&(right.seconds, right.nanos))),
        _ => None,
    }
}

fn evaluate(predicate: &AnswerPredicate, values: &HashMap<String, Value>) -> Result<Value> {
    use dokime_protocol::model::answer_predicate::Kind;
    let kind = match predicate.kind.as_ref() {
        Some(Kind::Literal(value)) => return Ok(value.clone()),
        Some(Kind::Field(field)) => {
            return values.get(field).cloned().ok_or_else(|| Error::Invalid(format!("Answer Prompt: Supply Rule Field {field}")));
        }
        Some(Kind::Not(child)) => ValueKind::Boolean(evaluate(child, values)?.kind == Some(ValueKind::Boolean(false))),
        Some(Kind::Binary(binary)) => {
            let left =
                evaluate(binary.left.as_ref().ok_or_else(|| Error::Invalid("Repair the Answer Rule's Left Operand".into()))?, values)?;
            let right =
                evaluate(binary.right.as_ref().ok_or_else(|| Error::Invalid("Repair the Answer Rule's Right Operand".into()))?, values)?;
            let ordering = compare(&left, &right);
            let result = match AnswerOperator::try_from(binary.operator).unwrap_or_default() {
                AnswerOperator::And => left.kind == Some(ValueKind::Boolean(true)) && right.kind == Some(ValueKind::Boolean(true)),
                AnswerOperator::Or => left.kind == Some(ValueKind::Boolean(true)) || right.kind == Some(ValueKind::Boolean(true)),
                AnswerOperator::Eq => left == right,
                AnswerOperator::Ne => left != right,
                AnswerOperator::Gt => ordering == Some(Ordering::Greater),
                AnswerOperator::Ge => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
                AnswerOperator::Lt => ordering == Some(Ordering::Less),
                AnswerOperator::Le => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
                operator => match (&left.kind, &right.kind, operator) {
                    (Some(ValueKind::Text(left)), Some(ValueKind::Text(right)), AnswerOperator::Contains) => left.contains(right),
                    (Some(ValueKind::Text(left)), Some(ValueKind::Text(right)), AnswerOperator::StartsWith) => left.starts_with(right),
                    (Some(ValueKind::Text(left)), Some(ValueKind::Text(right)), AnswerOperator::EndsWith) => left.ends_with(right),
                    (Some(ValueKind::List(left)), _, AnswerOperator::Contains) => left.values.contains(&right),
                    _ => return Err(Error::Invalid("Answer Prompt: Repair the Rule Operator or Operand Types".into())),
                },
            };
            ValueKind::Boolean(result)
        }
        None => return Err(Error::Invalid("Answer Prompt: Repair the Empty Answer Rule".into())),
    };
    Ok(Value { kind: Some(kind) })
}

fn condition_met(condition: &EvidenceCondition, snapshot: &ExecutionSnapshot) -> Result<bool> {
    use dokime_protocol::model::evidence_condition::Kind;
    Ok(match condition.kind.as_ref() {
        Some(Kind::All(group)) => {
            for condition in &group.conditions {
                if !condition_met(condition, snapshot)? {
                    return Ok(false);
                }
            }
            true
        }
        Some(Kind::Any(group)) => {
            for condition in &group.conditions {
                if condition_met(condition, snapshot)? {
                    return Ok(true);
                }
            }
            false
        }
        Some(Kind::Record(record)) => {
            use prost::Message;
            use prost_reflect::ReflectMessage;
            let catalog = Declarations::decode(snapshot.execution.catalog.as_slice())?;
            let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.unwrap_or_default())
                .map_err(|error| Error::Invalid(error.to_string()))?;
            let mut message_type = record.kind.clone();
            for section in &catalog.sections {
                if section.r#type == "dokime.model.v1.RecordContracts" {
                    for contract in RecordContracts::decode(section.payload.as_slice())?.records {
                        if contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == record.kind) {
                            message_type = contract.message;
                        }
                    }
                }
            }
            let descriptor = pool
                .get_message_by_name(&message_type)
                .ok_or_else(|| Error::Invalid("Answer Prompt: Declare the Condition's Record Type".into()))?;
            let records = crate::host::projection::records(snapshot, std::slice::from_ref(&record.kind), None)?;
            let mut matched = false;
            for captured in records {
                let message = prost_reflect::DynamicMessage::decode(descriptor.clone(), captured.payload.as_slice())?;
                if record.fields.iter().all(|(field, expected)| {
                    message
                        .descriptor()
                        .get_field_by_name(field)
                        .is_some_and(|field| crate::host::projection::value(message.get_field(&field).as_ref()) == *expected)
                }) {
                    matched = true;
                    break;
                }
            }
            matched
        }
        Some(Kind::Stream(stream)) => {
            let samples = crate::host::projection::samples(snapshot, std::slice::from_ref(&stream.stream), None)?;
            let value = samples.last().and_then(|sample| sample.values.as_ref()).and_then(|values| values.values.get(&stream.field));
            let operator = StreamMatchOperator::try_from(stream.operator).unwrap_or_default();
            if operator == StreamMatchOperator::Exists {
                return Ok(value.is_some());
            }
            let (Some(value), Some(target)) = (value, stream.target.as_ref()) else {
                return Ok(false);
            };
            match operator {
                StreamMatchOperator::Eq => value == target,
                StreamMatchOperator::Gt => compare(value, target) == Some(Ordering::Greater),
                StreamMatchOperator::Ge => {
                    matches!(compare(value, target), Some(Ordering::Greater | Ordering::Equal))
                }
                StreamMatchOperator::Lt => compare(value, target) == Some(Ordering::Less),
                StreamMatchOperator::Le => matches!(compare(value, target), Some(Ordering::Less | Ordering::Equal)),
                StreamMatchOperator::Within => match (&value.kind, &target.kind, stream.tolerance) {
                    (Some(ValueKind::Number(value)), Some(ValueKind::Number(target)), Some(tolerance)) => {
                        (value - target).abs() <= tolerance
                    }
                    _ => false,
                },
                _ => return Err(Error::Invalid("Answer Prompt: Repair the Stream Condition Operator".into())),
            }
        }
        None => false,
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::Error;
    use dokime_protocol::model::value::Kind;
    use dokime_protocol::model::{Metadata, Prompt, PromptAction, PromptField, PromptFieldValidation, Value, ValueType};
    use dokime_storage::{Execution, ExecutionSnapshot, ExecutionState};
    use std::collections::HashMap;

    #[test]
    fn operator_answers_preserve_integer_bounds_and_rejection_semantics() {
        let definition = Prompt {
            fields: vec![PromptField {
                metadata: Some(Metadata {
                    identifier: "count".into(),
                    ..Default::default()
                }),
                r#type: ValueType::UnsignedInteger as i32,
                required: true,
                validation: Some(PromptFieldValidation {
                    minimum: Some(Value {
                        kind: Some(Kind::UnsignedInteger(u64::MAX - 1)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            actions: vec![PromptAction {
                identifier: "submit".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let snapshot = ExecutionSnapshot {
            execution: Execution::default(),
            state: ExecutionState::Open,
            evidence: Vec::new(),
            samples: Vec::new(),
            seal: Vec::new(),
        };
        let mut values = HashMap::from([(
            "count".into(),
            Value {
                kind: Some(Kind::UnsignedInteger(u64::MAX)),
            },
        )]);
        assert!(validate(&definition, "submit", true, &values, &snapshot).is_ok());

        values.insert(
            "count".into(),
            Value {
                kind: Some(Kind::UnsignedInteger(u64::MAX - 2)),
            },
        );
        assert!(matches!(validate(&definition, "submit", true, &values, &snapshot), Err(Error::Invalid(_))));
        values.clear();
        assert!(matches!(validate(&definition, "submit", true, &values, &snapshot), Err(Error::Invalid(_))));
        assert!(validate(&definition, "submit", false, &values, &snapshot).is_ok());
        assert!(matches!(validate(&definition, "unknown", false, &values, &snapshot), Err(Error::Invalid(_))));
    }

    #[test]
    fn conditional_actions_refuse_without_required_evidence() {
        let definition = Prompt {
            actions: vec![PromptAction {
                identifier: "confirm".into(),
                requires_condition: true,
                ..Default::default()
            }],
            ..Default::default()
        };
        let snapshot = ExecutionSnapshot {
            execution: Execution::default(),
            state: ExecutionState::Open,
            evidence: Vec::new(),
            samples: Vec::new(),
            seal: Vec::new(),
        };
        let values = HashMap::new();
        assert!(matches!(validate(&definition, "confirm", true, &values, &snapshot), Err(Error::Precondition(_))));
        assert!(matches!(validate(&definition, "confirm", false, &values, &snapshot), Err(Error::Precondition(_))));
    }
}

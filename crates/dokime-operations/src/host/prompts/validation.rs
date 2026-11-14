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
use dokime_protocol::model;
use dokime_storage::ExecutionSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn validate(
    definition: &dokime_protocol::model::Prompt, action: &str, accepted: bool, values: &HashMap<String, dokime_protocol::model::Value>,
    snapshot: &ExecutionSnapshot,
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
            let Some(dokime_protocol::model::value::Kind::List(list)) = &value.kind else {
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
            if let Some(dokime_protocol::model::value::Kind::Text(text)) = &value.kind {
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
        if evaluate(predicate, values)?.kind != Some(dokime_protocol::model::value::Kind::Boolean(true)) {
            return Err(Error::Invalid(format!("Answer Prompt: {}", rule.message)));
        }
    }
    Ok(())
}

/* === Private Functions === */

pub(crate) fn type_matches(kind: i32, value: &dokime_protocol::model::Value) -> bool {
    use dokime_protocol::model::value::Kind;
    matches!(
        (dokime_protocol::model::ValueType::try_from(kind), &value.kind),
        (Ok(dokime_protocol::model::ValueType::Boolean), Some(Kind::Boolean(_)))
            | (Ok(dokime_protocol::model::ValueType::Integer), Some(Kind::Integer(_)))
            | (Ok(dokime_protocol::model::ValueType::UnsignedInteger), Some(Kind::UnsignedInteger(_)))
            | (Ok(dokime_protocol::model::ValueType::Number), Some(Kind::Number(_)))
            | (Ok(dokime_protocol::model::ValueType::Text), Some(Kind::Text(_)))
            | (Ok(dokime_protocol::model::ValueType::Bytes), Some(Kind::Data(_)))
            | (Ok(dokime_protocol::model::ValueType::List), Some(Kind::List(_)))
            | (Ok(dokime_protocol::model::ValueType::Datetime), Some(Kind::Datetime(_)))
    ) && !matches!(value.kind, Some(Kind::Number(number)) if !number.is_finite())
}

fn compare(left: &dokime_protocol::model::Value, right: &dokime_protocol::model::Value) -> Option<Ordering> {
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

fn evaluate(
    predicate: &dokime_protocol::model::AnswerPredicate, values: &HashMap<String, dokime_protocol::model::Value>,
) -> Result<dokime_protocol::model::Value> {
    use dokime_protocol::model::answer_predicate::Kind;
    use dokime_protocol::model::value::Kind as Value;
    let kind = match predicate.kind.as_ref() {
        Some(Kind::Literal(value)) => return Ok(value.clone()),
        Some(Kind::Field(field)) => {
            return values.get(field).cloned().ok_or_else(|| Error::Invalid(format!("Answer Prompt: Supply Rule Field {field}")));
        }
        Some(Kind::Not(child)) => Value::Boolean(evaluate(child, values)?.kind == Some(Value::Boolean(false))),
        Some(Kind::Binary(binary)) => {
            let left =
                evaluate(binary.left.as_ref().ok_or_else(|| Error::Invalid("Repair the Answer Rule's Left Operand".into()))?, values)?;
            let right =
                evaluate(binary.right.as_ref().ok_or_else(|| Error::Invalid("Repair the Answer Rule's Right Operand".into()))?, values)?;
            let ordering = compare(&left, &right);
            let result = match dokime_protocol::model::AnswerOperator::try_from(binary.operator).unwrap_or_default() {
                dokime_protocol::model::AnswerOperator::And => {
                    left.kind == Some(Value::Boolean(true)) && right.kind == Some(Value::Boolean(true))
                }
                dokime_protocol::model::AnswerOperator::Or => {
                    left.kind == Some(Value::Boolean(true)) || right.kind == Some(Value::Boolean(true))
                }
                dokime_protocol::model::AnswerOperator::Eq => left == right,
                dokime_protocol::model::AnswerOperator::Ne => left != right,
                dokime_protocol::model::AnswerOperator::Gt => ordering == Some(Ordering::Greater),
                dokime_protocol::model::AnswerOperator::Ge => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
                dokime_protocol::model::AnswerOperator::Lt => ordering == Some(Ordering::Less),
                dokime_protocol::model::AnswerOperator::Le => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
                operator => match (&left.kind, &right.kind, operator) {
                    (Some(Value::Text(left)), Some(Value::Text(right)), dokime_protocol::model::AnswerOperator::Contains) => {
                        left.contains(right)
                    }
                    (Some(Value::Text(left)), Some(Value::Text(right)), dokime_protocol::model::AnswerOperator::StartsWith) => {
                        left.starts_with(right)
                    }
                    (Some(Value::Text(left)), Some(Value::Text(right)), dokime_protocol::model::AnswerOperator::EndsWith) => {
                        left.ends_with(right)
                    }
                    (Some(Value::List(left)), _, dokime_protocol::model::AnswerOperator::Contains) => left.values.contains(&right),
                    _ => return Err(Error::Invalid("Answer Prompt: Repair the Rule Operator or Operand Types".into())),
                },
            };
            Value::Boolean(result)
        }
        None => return Err(Error::Invalid("Answer Prompt: Repair the Empty Answer Rule".into())),
    };
    Ok(dokime_protocol::model::Value { kind: Some(kind) })
}

fn condition_met(condition: &dokime_protocol::model::EvidenceCondition, snapshot: &ExecutionSnapshot) -> Result<bool> {
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
            let catalog = harmos_protocol::model::Declarations::decode(snapshot.execution.catalog.as_slice())?;
            let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.unwrap_or_default())
                .map_err(|error| Error::Invalid(error.to_string()))?;
            let mut message_type = record.kind.clone();
            for section in &catalog.sections {
                if section.r#type == "dokime.model.v1.RecordContracts" {
                    for contract in dokime_protocol::model::RecordContracts::decode(section.payload.as_slice())?.records {
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
            let operator = dokime_protocol::model::StreamMatchOperator::try_from(stream.operator).unwrap_or_default();
            if operator == dokime_protocol::model::StreamMatchOperator::Exists {
                return Ok(value.is_some());
            }
            let (Some(value), Some(target)) = (value, stream.target.as_ref()) else {
                return Ok(false);
            };
            match operator {
                dokime_protocol::model::StreamMatchOperator::Eq => value == target,
                dokime_protocol::model::StreamMatchOperator::Gt => compare(value, target) == Some(Ordering::Greater),
                dokime_protocol::model::StreamMatchOperator::Ge => {
                    matches!(compare(value, target), Some(Ordering::Greater | Ordering::Equal))
                }
                dokime_protocol::model::StreamMatchOperator::Lt => compare(value, target) == Some(Ordering::Less),
                dokime_protocol::model::StreamMatchOperator::Le => matches!(compare(value, target), Some(Ordering::Less | Ordering::Equal)),
                dokime_protocol::model::StreamMatchOperator::Within => match (&value.kind, &target.kind, stream.tolerance) {
                    (
                        Some(dokime_protocol::model::value::Kind::Number(value)),
                        Some(dokime_protocol::model::value::Kind::Number(target)),
                        Some(tolerance),
                    ) => (value - target).abs() <= tolerance,
                    _ => false,
                },
                _ => return Err(Error::Invalid("Answer Prompt: Repair the Stream Condition Operator".into())),
            }
        }
        None => false,
    })
}

/* === Tests === */

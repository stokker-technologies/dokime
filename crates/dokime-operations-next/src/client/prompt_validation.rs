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
use dokime_protocol::v1;
use dokime_storage::ExecutionSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn validate(
    definition: &v1::Prompt, action: &str, accepted: bool, values: &HashMap<String, v1::Value>, snapshot: &ExecutionSnapshot,
) -> Result<()> {
    let action = definition
        .actions
        .iter()
        .find(|candidate| candidate.identifier == action)
        .ok_or_else(|| Error::Invalid("Answer Prompt: Choose an Action Declared by the Question".into()))?;
    if action.requires_condition
        && !definition.condition.as_ref().map(|condition| condition_met(condition, snapshot)).transpose()?.unwrap_or(false)
    {
        return Err(Error::Conflict("Answer Prompt: Wait Until the Required Evidence Condition Is Satisfied".into()));
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
            let Some(v1::value::Kind::List(list)) = &value.kind else {
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
            if let Some(v1::value::Kind::Text(text)) = &value.kind {
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
        if evaluate(predicate, values)?.kind != Some(v1::value::Kind::Boolean(true)) {
            return Err(Error::Invalid(format!("Answer Prompt: {}", rule.message)));
        }
    }
    Ok(())
}

/* === Private Functions === */

pub(super) fn type_matches(kind: i32, value: &v1::Value) -> bool {
    use v1::value::Kind;
    matches!(
        (v1::ValueType::try_from(kind), &value.kind),
        (Ok(v1::ValueType::Boolean), Some(Kind::Boolean(_)))
            | (Ok(v1::ValueType::Integer), Some(Kind::Integer(_)))
            | (Ok(v1::ValueType::UnsignedInteger), Some(Kind::UnsignedInteger(_)))
            | (Ok(v1::ValueType::Number), Some(Kind::Number(_)))
            | (Ok(v1::ValueType::Text), Some(Kind::Text(_)))
            | (Ok(v1::ValueType::Bytes), Some(Kind::Data(_)))
            | (Ok(v1::ValueType::List), Some(Kind::List(_)))
            | (Ok(v1::ValueType::Datetime), Some(Kind::Datetime(_)))
    ) && !matches!(value.kind, Some(Kind::Number(number)) if !number.is_finite())
}

fn compare(left: &v1::Value, right: &v1::Value) -> Option<Ordering> {
    use v1::value::Kind;
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

fn evaluate(predicate: &v1::AnswerPredicate, values: &HashMap<String, v1::Value>) -> Result<v1::Value> {
    use v1::answer_predicate::Kind;
    use v1::value::Kind as Value;
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
            let result = match v1::AnswerOperator::try_from(binary.operator).unwrap_or_default() {
                v1::AnswerOperator::And => left.kind == Some(Value::Boolean(true)) && right.kind == Some(Value::Boolean(true)),
                v1::AnswerOperator::Or => left.kind == Some(Value::Boolean(true)) || right.kind == Some(Value::Boolean(true)),
                v1::AnswerOperator::Eq => left == right,
                v1::AnswerOperator::Ne => left != right,
                v1::AnswerOperator::Gt => ordering == Some(Ordering::Greater),
                v1::AnswerOperator::Ge => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
                v1::AnswerOperator::Lt => ordering == Some(Ordering::Less),
                v1::AnswerOperator::Le => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
                operator => match (&left.kind, &right.kind, operator) {
                    (Some(Value::Text(left)), Some(Value::Text(right)), v1::AnswerOperator::Contains) => left.contains(right),
                    (Some(Value::Text(left)), Some(Value::Text(right)), v1::AnswerOperator::StartsWith) => left.starts_with(right),
                    (Some(Value::Text(left)), Some(Value::Text(right)), v1::AnswerOperator::EndsWith) => left.ends_with(right),
                    (Some(Value::List(left)), _, v1::AnswerOperator::Contains) => left.values.contains(&right),
                    _ => return Err(Error::Invalid("Answer Prompt: Repair the Rule Operator or Operand Types".into())),
                },
            };
            Value::Boolean(result)
        }
        None => return Err(Error::Invalid("Answer Prompt: Repair the Empty Answer Rule".into())),
    };
    Ok(v1::Value { kind: Some(kind) })
}

fn condition_met(condition: &v1::EvidenceCondition, snapshot: &ExecutionSnapshot) -> Result<bool> {
    use v1::evidence_condition::Kind;
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
            let catalog = harmos_protocol::v1::Catalog::decode(snapshot.execution.catalog.as_slice())?;
            let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.unwrap_or_default())
                .map_err(|error| Error::Invalid(error.to_string()))?;
            let mut message_type = record.kind.clone();
            for section in &catalog.sections {
                if section.r#type == "dokime.v1.RecordContracts" {
                    for contract in v1::RecordContracts::decode(section.payload.as_slice())?.records {
                        if contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == record.kind) {
                            message_type = contract.message;
                        }
                    }
                }
            }
            let descriptor = pool
                .get_message_by_name(&message_type)
                .ok_or_else(|| Error::Invalid("Answer Prompt: Declare the Condition's Record Type".into()))?;
            let records = super::projection::records(snapshot, std::slice::from_ref(&record.kind), None)?;
            let mut matched = false;
            for captured in records {
                let message = prost_reflect::DynamicMessage::decode(descriptor.clone(), captured.payload.as_slice())?;
                if record.fields.iter().all(|(field, expected)| {
                    message
                        .descriptor()
                        .get_field_by_name(field)
                        .is_some_and(|field| super::projection::value(message.get_field(&field).as_ref()) == *expected)
                }) {
                    matched = true;
                    break;
                }
            }
            matched
        }
        Some(Kind::Stream(stream)) => {
            let samples = super::projection::samples(snapshot, std::slice::from_ref(&stream.stream), None)?;
            let value = samples.last().and_then(|sample| sample.values.as_ref()).and_then(|values| values.values.get(&stream.field));
            let operator = v1::StreamMatchOperator::try_from(stream.operator).unwrap_or_default();
            if operator == v1::StreamMatchOperator::Exists {
                return Ok(value.is_some());
            }
            let (Some(value), Some(target)) = (value, stream.target.as_ref()) else {
                return Ok(false);
            };
            match operator {
                v1::StreamMatchOperator::Eq => value == target,
                v1::StreamMatchOperator::Gt => compare(value, target) == Some(Ordering::Greater),
                v1::StreamMatchOperator::Ge => matches!(compare(value, target), Some(Ordering::Greater | Ordering::Equal)),
                v1::StreamMatchOperator::Lt => compare(value, target) == Some(Ordering::Less),
                v1::StreamMatchOperator::Le => matches!(compare(value, target), Some(Ordering::Less | Ordering::Equal)),
                v1::StreamMatchOperator::Within => match (&value.kind, &target.kind, stream.tolerance) {
                    (Some(v1::value::Kind::Number(value)), Some(v1::value::Kind::Number(target)), Some(tolerance)) => {
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

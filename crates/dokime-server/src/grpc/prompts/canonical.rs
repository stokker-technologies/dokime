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
use std::collections::BTreeMap;
// Third Party Imports
use tonic::Status;
// Local Imports
use super::value::canonical::timestamp;
use super::value::{self, Budget, canonical, projection};
use dokime_app::capabilities::prompts;
use dokime_app::core::{Value, ValueType};
use dokime_app::definition::RuntimeCondition;
use dokime_app::expressions::{Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp};
use dokime_app::telemetry::TelemetryConditionOperator;
use dokime_contract::operations::{common, prompts as contract, schema, value as value_wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn identity(value: &prompts::PromptDefinition) -> contract::PromptDefinitionIdentity {
    contract::PromptDefinitionIdentity {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        title: value.title.clone(),
    }
}

pub(super) fn definition(
    value: &prompts::PromptDefinition,
) -> Result<contract::PromptDefinition, Status> {
    definition_heap(value, &mut Budget::new())?;
    definition_with_budget(value, &mut Budget::new())
}

pub(crate) fn instance(
    value: &prompts::PromptInstance,
) -> Result<contract::PromptInstance, Status> {
    instance_memory(value)?;
    let mut budget = Budget::new();
    Ok(contract::PromptInstance {
        prompt_id: value.prompt_id.as_str().to_owned(),
        definition: Some(definition_with_budget(&value.definition, &mut budget)?),
        kind: match value.kind {
            prompts::PromptKind::Confirmation => contract::PromptKind::Confirmation,
            prompts::PromptKind::Form => contract::PromptKind::Form,
            prompts::PromptKind::Choice => contract::PromptKind::Choice,
            prompts::PromptKind::Status => contract::PromptKind::Status,
            prompts::PromptKind::Verification => contract::PromptKind::Verification,
        },
        values: Some(object(&value.values, &mut budget, 0)?),
        scope: Some(canonical::scope(&value.scope)?),
        created_at: Some(timestamp(&value.created_at)?),
    })
}

pub(super) fn response(
    value: &prompts::PromptResponse,
) -> Result<contract::PromptResponse, Status> {
    Ok(contract::PromptResponse {
        prompt_id: value.prompt_id.as_str().to_owned(),
        action: value.action.clone(),
        responder: value.responder.clone(),
        submitted_at: Some(timestamp(&value.submitted_at)?),
        values: Some(object(&value.values, &mut Budget::new(), 0)?),
        accepted: value.accepted,
    })
}

pub(crate) fn instance_memory(source: &prompts::PromptInstance) -> Result<usize, Status> {
    let mut budget = Budget::new();
    timestamp(&source.created_at)?;
    let bytes = std::mem::size_of::<contract::PromptInstance>()
        .saturating_add(source.prompt_id.as_str().len())
        .saturating_add(definition_heap(&source.definition, &mut budget)?)
        .saturating_add(object_heap(&source.values, &mut budget, 0)?)
        .saturating_add(scope_heap(&source.scope)?);
    bounded(bytes)
}

pub(super) fn definition_memory(source: &prompts::PromptDefinition) -> Result<usize, Status> {
    bounded(
        std::mem::size_of::<contract::PromptDefinition>()
            .saturating_add(definition_heap(source, &mut Budget::new())?),
    )
}
pub(super) fn response_memory(source: &prompts::PromptResponse) -> Result<usize, Status> {
    timestamp(&source.submitted_at)?;
    for text in [
        source.prompt_id.as_str(),
        source.action.as_str(),
        source.responder.as_str(),
    ] {
        value::text(text)?;
    }
    bounded(
        std::mem::size_of::<contract::PromptResponse>()
            .saturating_add(source.prompt_id.as_str().len())
            .saturating_add(source.action.len())
            .saturating_add(source.responder.len())
            .saturating_add(object_heap(&source.values, &mut Budget::new(), 0)?),
    )
}

/* === Private Functions === */
fn definition_with_budget(
    value: &prompts::PromptDefinition,
    budget: &mut Budget,
) -> Result<contract::PromptDefinition, Status> {
    for len in [
        value.fields.len(),
        value.actions.len(),
        value.rules.len(),
        value.examples.len(),
    ] {
        value::collection(len)?;
    }
    value::text(&value.title)?;
    if let Some(message) = &value.message {
        value::text(message)?;
    }
    Ok(contract::PromptDefinition {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        archetype: match value.archetype {
            prompts::PromptArchetype::ConfirmationPrompt => contract::PromptArchetype::Confirmation,
            prompts::PromptArchetype::FormPrompt => contract::PromptArchetype::Form,
            prompts::PromptArchetype::ChoicePrompt => contract::PromptArchetype::Choice,
            prompts::PromptArchetype::StatusPrompt => contract::PromptArchetype::Status,
        },
        title: value.title.clone(),
        message: value.message.clone(),
        fields: collect(value.fields.iter().map(|field| field_wire(field, budget)))?,
        actions: collect(value.actions.iter().map(|action| {
            budget.node(0)?;
            value::text(&action.id)?;
            value::text(&action.label)?;
            Ok(contract::PromptAction {
                id: action.id.clone(),
                label: action.label.clone(),
                emphasis: match action.emphasis {
                    prompts::PromptActionEmphasis::Primary => {
                        contract::PromptActionEmphasis::Primary
                    }
                    prompts::PromptActionEmphasis::Secondary => {
                        contract::PromptActionEmphasis::Secondary
                    }
                    prompts::PromptActionEmphasis::Danger => contract::PromptActionEmphasis::Danger,
                },
                requires_condition: action.requires_condition,
            })
        }))?,
        rules: collect(value.rules.iter().map(|rule| {
            value::text(&rule.message)?;
            Ok(contract::AnswerRule {
                predicate: Some(predicate(&rule.predicate.expression, budget, 0)?),
                message: rule.message.clone(),
            })
        }))?,
        condition: value
            .condition
            .as_ref()
            .map(|condition| condition_wire(condition, budget, 0))
            .transpose()?,
        owner_suite_id: value.owner.as_ref().map(|id| id.as_str().to_owned()),
        template_identifier: value
            .template_identifier
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        examples: collect(
            value
                .examples
                .iter()
                .map(|values| object(values, budget, 0)),
        )?,
        level: crate::grpc::evidence::canonical::severity(value.level),
    })
}

fn field_wire(
    field: &prompts::PromptField,
    budget: &mut Budget,
) -> Result<contract::PromptField, Status> {
    budget.node(0)?;
    value::collection(field.choices.len())?;
    for choice in &field.choices {
        value::text(choice)?;
    }
    let validation = &field.validation;
    if let Some(pattern) = &validation.pattern {
        value::text(pattern)?;
    }
    Ok(contract::PromptField {
        metadata: Some(metadata(&field.metadata)?),
        value_type: match field.value_type {
            ValueType::Boolean => schema::ValueType::Boolean,
            ValueType::Integer => schema::ValueType::Integer,
            ValueType::Number => schema::ValueType::Number,
            ValueType::Text => schema::ValueType::Text,
            ValueType::Datetime => schema::ValueType::Datetime,
            ValueType::File => schema::ValueType::File,
            ValueType::Object => schema::ValueType::Object,
            ValueType::Array => schema::ValueType::Array,
        },
        kind: match field.kind {
            prompts::PromptFieldKind::Text => contract::PromptFieldKind::Text,
            prompts::PromptFieldKind::Float => contract::PromptFieldKind::Float,
            prompts::PromptFieldKind::Int => contract::PromptFieldKind::Int,
            prompts::PromptFieldKind::Choice => contract::PromptFieldKind::Choice,
            prompts::PromptFieldKind::Boolean => contract::PromptFieldKind::Boolean,
        },
        multiple: field.multiple,
        required: field.required,
        default_value: field
            .default
            .as_ref()
            .map(|v| project_value(v, budget, 0))
            .transpose()?,
        choices: collect(
            field
                .choices
                .iter()
                .map(|value| projection::text(value).map_err(failure)),
        )?,
        validation: Some(contract::PromptFieldValidation {
            min_length: validation.min_length.map(|v| v as u64),
            max_length: validation.max_length.map(|v| v as u64),
            pattern: validation.pattern.clone(),
            minimum: validation
                .minimum
                .as_ref()
                .map(|v| project_value(v, budget, 0))
                .transpose()?,
            maximum: validation
                .maximum
                .as_ref()
                .map(|v| project_value(v, budget, 0))
                .transpose()?,
            min_items: validation.min_items.map(|v| v as u64),
            max_items: validation.max_items.map(|v| v as u64),
        }),
    })
}

fn predicate(
    expression: &Expression,
    budget: &mut Budget,
    depth: usize,
) -> Result<contract::AnswerPredicate, Status> {
    let mut nodes = Vec::new();
    push_predicate(expression, budget, depth, &mut nodes)?;
    Ok(contract::AnswerPredicate { nodes })
}
fn push_predicate(
    expression: &Expression,
    budget: &mut Budget,
    depth: usize,
    nodes: &mut Vec<contract::AnswerPredicateNode>,
) -> Result<u32, Status> {
    use contract::AnswerPredicateKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len())
        .map_err(|_| Status::resource_exhausted("prompt predicate index overflow"))?;
    nodes.push(contract::AnswerPredicateNode { kind: None });
    let kind = match &expression.kind {
        ExpressionKind::Literal { value } => {
            Kind::Literal(project_value(value, budget, depth + 1)?)
        }
        ExpressionKind::Selector { path } => {
            value::text(path)?;
            Kind::Field(path.clone())
        }
        ExpressionKind::Unary {
            op: ExpressionUnaryOp::Not,
            value,
        } => Kind::Not(push_predicate(value, budget, depth + 1, nodes)?),
        ExpressionKind::Binary { op, left, right } => {
            Kind::Binary(contract::AnswerBinaryPredicate {
                op: match op {
                    ExpressionBinaryOp::And => contract::AnswerOperator::And,
                    ExpressionBinaryOp::Or => contract::AnswerOperator::Or,
                    ExpressionBinaryOp::Eq => contract::AnswerOperator::Eq,
                    ExpressionBinaryOp::Ne => contract::AnswerOperator::Ne,
                    ExpressionBinaryOp::Gt => contract::AnswerOperator::Gt,
                    ExpressionBinaryOp::Gte => contract::AnswerOperator::Gte,
                    ExpressionBinaryOp::Lt => contract::AnswerOperator::Lt,
                    ExpressionBinaryOp::Lte => contract::AnswerOperator::Lte,
                    ExpressionBinaryOp::Contains => contract::AnswerOperator::Contains,
                    ExpressionBinaryOp::StartsWith => contract::AnswerOperator::StartsWith,
                    ExpressionBinaryOp::EndsWith => contract::AnswerOperator::EndsWith,
                },
                left: push_predicate(left, budget, depth + 1, nodes)?,
                right: push_predicate(right, budget, depth + 1, nodes)?,
            })
        }
        ExpressionKind::Call { .. } => {
            return Err(Status::internal(
                "admitted prompt contains unsupported answer call",
            ));
        }
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}
fn condition_wire(
    condition: &RuntimeCondition,
    budget: &mut Budget,
    depth: usize,
) -> Result<contract::EvidenceCondition, Status> {
    let mut nodes = Vec::new();
    push_condition(condition, budget, depth, &mut nodes)?;
    Ok(contract::EvidenceCondition { nodes })
}
fn push_condition(
    condition: &RuntimeCondition,
    budget: &mut Budget,
    depth: usize,
    nodes: &mut Vec<contract::EvidenceConditionNode>,
) -> Result<u32, Status> {
    use contract::EvidenceConditionKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len())
        .map_err(|_| Status::resource_exhausted("prompt condition index overflow"))?;
    nodes.push(contract::EvidenceConditionNode { kind: None });
    let kind = match condition {
        RuntimeCondition::Record { condition } => Kind::Record(contract::RecordMatchCondition {
            record_type: condition.record_type.as_str().to_owned(),
            match_fields: Some(object(&condition.match_fields, budget, depth + 1)?),
        }),
        RuntimeCondition::Telemetry { condition } => {
            value::text(&condition.field)?;
            if condition.tolerance.is_some_and(|v| !v.is_finite()) {
                return Err(Status::internal("admitted prompt has nonfinite tolerance"));
            }
            Kind::Telemetry(contract::TelemetryMatchCondition {
                stream: condition.stream.as_str().to_owned(),
                field: condition.field.clone(),
                op: match condition.operator {
                    TelemetryConditionOperator::Exists => contract::TelemetryMatchOperator::Exists,
                    TelemetryConditionOperator::Eq => contract::TelemetryMatchOperator::Eq,
                    TelemetryConditionOperator::Gt => contract::TelemetryMatchOperator::Gt,
                    TelemetryConditionOperator::Gte => contract::TelemetryMatchOperator::Gte,
                    TelemetryConditionOperator::Lt => contract::TelemetryMatchOperator::Lt,
                    TelemetryConditionOperator::Lte => contract::TelemetryMatchOperator::Lte,
                    TelemetryConditionOperator::Within => contract::TelemetryMatchOperator::Within,
                },
                target: condition
                    .target
                    .as_ref()
                    .map(|v| project_value(v, budget, depth + 1))
                    .transpose()?,
                tolerance: condition.tolerance,
            })
        }
        RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } => {
            value::collection(conditions.len())?;
            let mut links = projection::slots(conditions.len()).map_err(failure)?;
            for child in conditions {
                links.push(push_condition(child, budget, depth + 1, nodes)?);
            }
            let group = contract::EvidenceConditions { conditions: links };
            if matches!(condition, RuntimeCondition::All { .. }) {
                Kind::All(group)
            } else {
                Kind::Any(group)
            }
        }
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}

fn object(
    values: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<value_wire::Value, Status> {
    budget.node(depth)?;
    canonical::object_at(values, budget, depth)
}

fn bounded(bytes: usize) -> Result<usize, Status> {
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "canonical prompt construction overflows",
        ));
    }
    Ok(bytes)
}

fn failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("canonical prompt construction exceeds its bound")
        }
        _ => Status::internal("canonical prompt projection failed"),
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = projection::slots(source.len()).map_err(failure)?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn metadata(source: &dokime_app::core::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<value_wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn scope_heap(source: &dokime_app::core::RecordScope) -> Result<usize, Status> {
    Ok(canonical::scope_memory(source)? - std::mem::size_of::<common::EvidenceScope>())
}

fn object_heap(
    source: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<usize, Status> {
    budget.node(depth)?;
    Ok(canonical::object_memory_at(source, budget, depth)?
        - std::mem::size_of::<value_wire::Value>())
}

fn value_heap(source: &Value, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    Ok(canonical::memory(source, budget, depth)? - std::mem::size_of::<value_wire::Value>())
}

fn slots<T>(length: usize) -> usize {
    length
        .saturating_mul(4)
        .saturating_mul(std::mem::size_of::<T>())
}

fn definition_heap(
    source: &prompts::PromptDefinition,
    budget: &mut Budget,
) -> Result<usize, Status> {
    for length in [
        source.fields.len(),
        source.actions.len(),
        source.rules.len(),
        source.examples.len(),
    ] {
        value::collection(length)?;
    }
    value::text(&source.title)?;
    if let Some(message) = &source.message {
        value::text(message)?;
    }
    let mut bytes = source
        .prompt_type
        .as_str()
        .len()
        .saturating_add(source.title.len())
        .saturating_add(source.message.as_ref().map_or(0, String::len))
        .saturating_add(source.owner.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(
            source
                .template_identifier
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(slots::<contract::PromptField>(source.fields.len()))
        .saturating_add(slots::<contract::PromptAction>(source.actions.len()))
        .saturating_add(slots::<contract::AnswerRule>(source.rules.len()))
        .saturating_add(slots::<value_wire::Value>(source.examples.len()));
    for field in &source.fields {
        budget.node(0)?;
        value::collection(field.choices.len())?;
        bytes = bytes
            .saturating_add(
                canonical::metadata_memory(&field.metadata)?
                    - std::mem::size_of::<common::Metadata>(),
            )
            .saturating_add(slots::<String>(field.choices.len()));
        for choice in &field.choices {
            value::text(choice)?;
            bytes = bytes.saturating_add(choice.len());
        }
        if let Some(pattern) = &field.validation.pattern {
            value::text(pattern)?;
            bytes = bytes.saturating_add(pattern.len());
        }
        for value in [
            &field.default,
            &field.validation.minimum,
            &field.validation.maximum,
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(value_heap(value, budget, 0)?);
        }
    }
    for action in &source.actions {
        budget.node(0)?;
        value::text(&action.id)?;
        value::text(&action.label)?;
        bytes = bytes
            .saturating_add(action.id.len())
            .saturating_add(action.label.len());
    }
    for rule in &source.rules {
        value::text(&rule.message)?;
        bytes = bytes
            .saturating_add(rule.message.len())
            .saturating_add(predicate_heap(&rule.predicate.expression, budget, 0)?);
    }
    if let Some(condition) = &source.condition {
        bytes = bytes.saturating_add(condition_heap(condition, budget, 0)?);
    }
    for example in &source.examples {
        bytes = bytes.saturating_add(object_heap(example, budget, 0)?);
    }
    bounded(bytes)
}

fn predicate_heap(source: &Expression, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    budget.node(depth)?;
    let bytes = match &source.kind {
        ExpressionKind::Literal { value } => value_heap(value, budget, depth + 1)?,
        ExpressionKind::Selector { path } => {
            value::text(path)?;
            path.len()
        }
        ExpressionKind::Unary {
            op: ExpressionUnaryOp::Not,
            value,
        } => predicate_heap(value, budget, depth + 1)?,
        ExpressionKind::Binary { left, right, .. } => predicate_heap(left, budget, depth + 1)?
            .saturating_add(predicate_heap(right, budget, depth + 1)?),
        ExpressionKind::Call { .. } => {
            return Err(Status::internal(
                "admitted prompt contains unsupported answer call",
            ));
        }
    };
    bounded(bytes.saturating_add(4 * std::mem::size_of::<contract::AnswerPredicateNode>()))
}

fn condition_heap(
    source: &RuntimeCondition,
    budget: &mut Budget,
    depth: usize,
) -> Result<usize, Status> {
    budget.node(depth)?;
    let bytes = match source {
        RuntimeCondition::Record { condition } => condition
            .record_type
            .as_str()
            .len()
            .saturating_add(object_heap(&condition.match_fields, budget, depth + 1)?),
        RuntimeCondition::Telemetry { condition } => {
            value::text(&condition.field)?;
            if condition.tolerance.is_some_and(|value| !value.is_finite()) {
                return Err(Status::internal("admitted prompt has nonfinite tolerance"));
            }
            let mut bytes = condition
                .stream
                .as_str()
                .len()
                .saturating_add(condition.field.len());
            if let Some(target) = &condition.target {
                bytes = bytes.saturating_add(value_heap(target, budget, depth + 1)?);
            }
            bytes
        }
        RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } => {
            value::collection(conditions.len())?;
            let mut bytes = slots::<u32>(conditions.len());
            for condition in conditions {
                bytes = bytes.saturating_add(condition_heap(condition, budget, depth + 1)?);
            }
            bytes
        }
    };
    bounded(bytes.saturating_add(4 * std::mem::size_of::<contract::EvidenceConditionNode>()))
}

/* === Tests === */

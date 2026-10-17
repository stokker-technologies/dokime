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
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;
use dokime_runtime::definition::RuntimeCondition;
use dokime_runtime::domains::prompts;
use dokime_runtime::expressions::{Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp};
use dokime_runtime::telemetry::TelemetryConditionOperator;
use dokime_domain::model::identity::wire as common;
use dokime_domain::operations::prompts as contract;
use dokime_domain::model::definition::wire as schema;
use dokime_domain::model::values::wire as value_wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn identity(value: &prompts::PromptDefinition) -> dokime_domain::model::prompts::wire::PromptDefinitionIdentity {
    dokime_domain::model::prompts::wire::PromptDefinitionIdentity {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        title: value.title.clone(),
    }
}

pub(super) fn definition(
    value: &prompts::PromptDefinition,
) -> Result<dokime_domain::model::prompts::wire::PromptDefinition, Status> {
    definition_heap(value, &mut Budget::new())?;
    definition_with_budget(value, &mut Budget::new())
}

pub(crate) fn instance(
    value: &prompts::PromptInstance,
) -> Result<dokime_domain::model::prompts::wire::PromptInstance, Status> {
    instance_memory(value)?;
    let mut budget = Budget::new();
    Ok(dokime_domain::model::prompts::wire::PromptInstance {
        prompt_id: value.prompt_id.as_str().to_owned(),
        definition: Some(definition_with_budget(&value.definition, &mut budget)?),
        kind: match value.kind {
            prompts::PromptKind::Confirmation => dokime_domain::model::prompts::wire::PromptKind::Confirmation,
            prompts::PromptKind::Form => dokime_domain::model::prompts::wire::PromptKind::Form,
            prompts::PromptKind::Choice => dokime_domain::model::prompts::wire::PromptKind::Choice,
            prompts::PromptKind::Status => dokime_domain::model::prompts::wire::PromptKind::Status,
            prompts::PromptKind::Verification => dokime_domain::model::prompts::wire::PromptKind::Verification,
        },
        values: Some(object(&value.values, &mut budget, 0)?),
        scope: Some(canonical::scope(&value.scope)?),
        created_at: Some(timestamp(&value.created_at)?),
    })
}

pub(super) fn response(
    value: &prompts::PromptResponse,
) -> Result<dokime_domain::model::prompts::wire::PromptResponse, Status> {
    Ok(dokime_domain::model::prompts::wire::PromptResponse {
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
    let bytes = std::mem::size_of::<dokime_domain::model::prompts::wire::PromptInstance>()
        .saturating_add(source.prompt_id.as_str().len())
        .saturating_add(definition_heap(&source.definition, &mut budget)?)
        .saturating_add(object_heap(&source.values, &mut budget, 0)?)
        .saturating_add(scope_heap(&source.scope)?);
    bounded(bytes)
}

/* === Private Functions === */
fn definition_with_budget(
    value: &prompts::PromptDefinition,
    budget: &mut Budget,
) -> Result<dokime_domain::model::prompts::wire::PromptDefinition, Status> {
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
    Ok(dokime_domain::model::prompts::wire::PromptDefinition {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        archetype: match value.archetype {
            prompts::PromptArchetype::ConfirmationPrompt => dokime_domain::model::prompts::wire::PromptArchetype::Confirmation,
            prompts::PromptArchetype::FormPrompt => dokime_domain::model::prompts::wire::PromptArchetype::Form,
            prompts::PromptArchetype::ChoicePrompt => dokime_domain::model::prompts::wire::PromptArchetype::Choice,
            prompts::PromptArchetype::StatusPrompt => dokime_domain::model::prompts::wire::PromptArchetype::Status,
        },
        title: value.title.clone(),
        message: value.message.clone(),
        fields: collect(value.fields.iter().map(|field| field_wire(field, budget)))?,
        actions: collect(value.actions.iter().map(|action| {
            budget.node(0)?;
            value::text(&action.id)?;
            value::text(&action.label)?;
            Ok(dokime_domain::model::prompts::wire::PromptAction {
                id: action.id.clone(),
                label: action.label.clone(),
                emphasis: match action.emphasis {
                    prompts::PromptActionEmphasis::Primary => {
                        dokime_domain::model::prompts::wire::PromptActionEmphasis::Primary
                    }
                    prompts::PromptActionEmphasis::Secondary => {
                        dokime_domain::model::prompts::wire::PromptActionEmphasis::Secondary
                    }
                    prompts::PromptActionEmphasis::Danger => dokime_domain::model::prompts::wire::PromptActionEmphasis::Danger,
                },
                requires_condition: action.requires_condition,
            })
        }))?,
        rules: collect(value.rules.iter().map(|rule| {
            value::text(&rule.message)?;
            Ok(dokime_domain::model::prompts::wire::AnswerRule {
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
) -> Result<dokime_domain::model::prompts::wire::PromptField, Status> {
    budget.node(0)?;
    value::collection(field.choices.len())?;
    for choice in &field.choices {
        value::text(choice)?;
    }
    let validation = &field.validation;
    if let Some(pattern) = &validation.pattern {
        value::text(pattern)?;
    }
    Ok(dokime_domain::model::prompts::wire::PromptField {
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
            prompts::PromptFieldKind::Text => dokime_domain::model::prompts::wire::PromptFieldKind::Text,
            prompts::PromptFieldKind::Float => dokime_domain::model::prompts::wire::PromptFieldKind::Float,
            prompts::PromptFieldKind::Int => dokime_domain::model::prompts::wire::PromptFieldKind::Int,
            prompts::PromptFieldKind::Choice => dokime_domain::model::prompts::wire::PromptFieldKind::Choice,
            prompts::PromptFieldKind::Boolean => dokime_domain::model::prompts::wire::PromptFieldKind::Boolean,
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
        validation: Some(dokime_domain::model::prompts::wire::PromptFieldValidation {
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
) -> Result<dokime_domain::model::prompts::wire::AnswerPredicate, Status> {
    let mut nodes = Vec::new();
    push_predicate(expression, budget, depth, &mut nodes)?;
    Ok(dokime_domain::model::prompts::wire::AnswerPredicate { nodes })
}
fn push_predicate(
    expression: &Expression,
    budget: &mut Budget,
    depth: usize,
    nodes: &mut Vec<dokime_domain::model::prompts::wire::AnswerPredicateNode>,
) -> Result<u32, Status> {
    use dokime_domain::model::prompts::wire::AnswerPredicateKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len())
        .map_err(|_| Status::resource_exhausted("prompt predicate index overflow"))?;
    nodes.push(dokime_domain::model::prompts::wire::AnswerPredicateNode { kind: None });
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
            Kind::Binary(dokime_domain::model::prompts::wire::AnswerBinaryPredicate {
                op: match op {
                    ExpressionBinaryOp::And => dokime_domain::model::prompts::wire::AnswerOperator::And,
                    ExpressionBinaryOp::Or => dokime_domain::model::prompts::wire::AnswerOperator::Or,
                    ExpressionBinaryOp::Eq => dokime_domain::model::prompts::wire::AnswerOperator::Eq,
                    ExpressionBinaryOp::Ne => dokime_domain::model::prompts::wire::AnswerOperator::Ne,
                    ExpressionBinaryOp::Gt => dokime_domain::model::prompts::wire::AnswerOperator::Gt,
                    ExpressionBinaryOp::Gte => dokime_domain::model::prompts::wire::AnswerOperator::Gte,
                    ExpressionBinaryOp::Lt => dokime_domain::model::prompts::wire::AnswerOperator::Lt,
                    ExpressionBinaryOp::Lte => dokime_domain::model::prompts::wire::AnswerOperator::Lte,
                    ExpressionBinaryOp::Contains => dokime_domain::model::prompts::wire::AnswerOperator::Contains,
                    ExpressionBinaryOp::StartsWith => dokime_domain::model::prompts::wire::AnswerOperator::StartsWith,
                    ExpressionBinaryOp::EndsWith => dokime_domain::model::prompts::wire::AnswerOperator::EndsWith,
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
) -> Result<dokime_domain::model::prompts::wire::EvidenceCondition, Status> {
    let mut nodes = Vec::new();
    push_condition(condition, budget, depth, &mut nodes)?;
    Ok(dokime_domain::model::prompts::wire::EvidenceCondition { nodes })
}
fn push_condition(
    condition: &RuntimeCondition,
    budget: &mut Budget,
    depth: usize,
    nodes: &mut Vec<dokime_domain::model::prompts::wire::EvidenceConditionNode>,
) -> Result<u32, Status> {
    use dokime_domain::model::prompts::wire::EvidenceConditionKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len())
        .map_err(|_| Status::resource_exhausted("prompt condition index overflow"))?;
    nodes.push(dokime_domain::model::prompts::wire::EvidenceConditionNode { kind: None });
    let kind = match condition {
        RuntimeCondition::Record { condition } => Kind::Record(dokime_domain::model::prompts::wire::RecordMatchCondition {
            record_type: condition.record_type.as_str().to_owned(),
            match_fields: Some(object(&condition.match_fields, budget, depth + 1)?),
        }),
        RuntimeCondition::Telemetry { condition } => {
            value::text(&condition.field)?;
            if condition.tolerance.is_some_and(|v| !v.is_finite()) {
                return Err(Status::internal("admitted prompt has nonfinite tolerance"));
            }
            Kind::Telemetry(dokime_domain::model::prompts::wire::TelemetryMatchCondition {
                stream: condition.stream.as_str().to_owned(),
                field: condition.field.clone(),
                op: match condition.operator {
                    TelemetryConditionOperator::Exists => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Exists,
                    TelemetryConditionOperator::Eq => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Eq,
                    TelemetryConditionOperator::Gt => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Gt,
                    TelemetryConditionOperator::Gte => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Gte,
                    TelemetryConditionOperator::Lt => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Lt,
                    TelemetryConditionOperator::Lte => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Lte,
                    TelemetryConditionOperator::Within => dokime_domain::model::prompts::wire::TelemetryMatchOperator::Within,
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
            let group = dokime_domain::model::prompts::wire::EvidenceConditions { conditions: links };
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
) -> Result<dokime_domain::model::values::wire::Value, Status> {
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

fn metadata(source: &dokime_domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &Value,
    budget: &mut Budget,
    depth: usize,
) -> Result<dokime_domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn scope_heap(source: &dokime_domain::model::execution::RecordScope) -> Result<usize, Status> {
    Ok(canonical::scope_memory(source)? - std::mem::size_of::<common::EvidenceScope>())
}

fn object_heap(
    source: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<usize, Status> {
    budget.node(depth)?;
    Ok(canonical::object_memory_at(source, budget, depth)?
        - std::mem::size_of::<dokime_domain::model::values::wire::Value>())
}

fn value_heap(source: &Value, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    Ok(canonical::memory(source, budget, depth)? - std::mem::size_of::<dokime_domain::model::values::wire::Value>())
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
        .saturating_add(slots::<dokime_domain::model::prompts::wire::PromptField>(source.fields.len()))
        .saturating_add(slots::<dokime_domain::model::prompts::wire::PromptAction>(source.actions.len()))
        .saturating_add(slots::<dokime_domain::model::prompts::wire::AnswerRule>(source.rules.len()))
        .saturating_add(slots::<dokime_domain::model::values::wire::Value>(source.examples.len()));
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
    bounded(bytes.saturating_add(4 * std::mem::size_of::<dokime_domain::model::prompts::wire::AnswerPredicateNode>()))
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
    bounded(bytes.saturating_add(4 * std::mem::size_of::<dokime_domain::model::prompts::wire::EvidenceConditionNode>()))
}

/* === Tests === */

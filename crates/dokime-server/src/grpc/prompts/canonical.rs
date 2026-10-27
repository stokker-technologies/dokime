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
use super::value::{Budget, canonical, projection};
use dokime::definition::RuntimeCondition;
use dokime::domain::model::definition::wire as schema;
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::values::Value;
use dokime::domain::model::values::ValueType;
use dokime::domains::prompts;
use dokime::expressions::{Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp};
use dokime::telemetry::TelemetryConditionOperator;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn identity(value: &prompts::PromptDefinition) -> dokime::domain::model::prompts::wire::PromptDefinitionIdentity {
    dokime::domain::model::prompts::wire::PromptDefinitionIdentity {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        title: value.title.clone(),
    }
}

pub(super) fn definition(value: &prompts::PromptDefinition) -> Result<dokime::domain::model::prompts::wire::PromptDefinition, Status> {
    definition_with_budget(value, &mut Budget::new()?)
}

pub(crate) fn instance(value: &prompts::PromptInstance) -> Result<dokime::domain::model::prompts::wire::PromptInstance, Status> {
    let mut budget = Budget::new()?;
    Ok(dokime::domain::model::prompts::wire::PromptInstance {
        prompt_id: value.prompt_id.as_str().to_owned(),
        definition: Some(definition_with_budget(&value.definition, &mut budget)?),
        kind: match value.kind {
            prompts::PromptKind::Confirmation => dokime::domain::model::prompts::wire::PromptKind::Confirmation,
            prompts::PromptKind::Form => dokime::domain::model::prompts::wire::PromptKind::Form,
            prompts::PromptKind::Choice => dokime::domain::model::prompts::wire::PromptKind::Choice,
            prompts::PromptKind::Status => dokime::domain::model::prompts::wire::PromptKind::Status,
            prompts::PromptKind::Verification => dokime::domain::model::prompts::wire::PromptKind::Verification,
        },
        values: Some(object(&value.values, &mut budget, 0)?),
        scope: Some(canonical::scope(&value.scope)?),
        created_at: Some(timestamp(&value.created_at)?),
    })
}

pub(super) fn response(value: &prompts::PromptResponse) -> Result<dokime::domain::model::prompts::wire::PromptResponse, Status> {
    Ok(dokime::domain::model::prompts::wire::PromptResponse {
        prompt_id: value.prompt_id.as_str().to_owned(),
        action: value.action.clone(),
        responder: value.responder.clone(),
        submitted_at: Some(timestamp(&value.submitted_at)?),
        values: Some(object(&value.values, &mut Budget::new()?, 0)?),
        accepted: value.accepted,
    })
}

/* === Private Functions === */

fn definition_with_budget(
    value: &prompts::PromptDefinition, budget: &mut Budget,
) -> Result<dokime::domain::model::prompts::wire::PromptDefinition, Status> {
    Ok(dokime::domain::model::prompts::wire::PromptDefinition {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        archetype: match value.archetype {
            prompts::PromptArchetype::ConfirmationPrompt => dokime::domain::model::prompts::wire::PromptArchetype::Confirmation,
            prompts::PromptArchetype::FormPrompt => dokime::domain::model::prompts::wire::PromptArchetype::Form,
            prompts::PromptArchetype::ChoicePrompt => dokime::domain::model::prompts::wire::PromptArchetype::Choice,
            prompts::PromptArchetype::StatusPrompt => dokime::domain::model::prompts::wire::PromptArchetype::Status,
        },
        title: value.title.clone(),
        message: value.message.clone(),
        fields: collect(value.fields.iter().map(|field| field_wire(field, budget)))?,
        actions: collect(value.actions.iter().map(|action| {
            budget.node(0)?;
            Ok(dokime::domain::model::prompts::wire::PromptAction {
                id: action.id.clone(),
                label: action.label.clone(),
                emphasis: match action.emphasis {
                    prompts::PromptActionEmphasis::Primary => dokime::domain::model::prompts::wire::PromptActionEmphasis::Primary,
                    prompts::PromptActionEmphasis::Secondary => dokime::domain::model::prompts::wire::PromptActionEmphasis::Secondary,
                    prompts::PromptActionEmphasis::Danger => dokime::domain::model::prompts::wire::PromptActionEmphasis::Danger,
                },
                requires_condition: action.requires_condition,
            })
        }))?,
        rules: collect(value.rules.iter().map(|rule| {
            Ok(dokime::domain::model::prompts::wire::AnswerRule {
                predicate: Some(predicate(&rule.predicate.expression, budget, 0)?),
                message: rule.message.clone(),
            })
        }))?,
        condition: value.condition.as_ref().map(|condition| condition_wire(condition, budget, 0)).transpose()?,
        owner_suite_id: value.owner.as_ref().map(|id| id.as_str().to_owned()),
        template_identifier: value.template_identifier.as_ref().map(|id| id.as_str().to_owned()),
        examples: collect(value.examples.iter().map(|values| object(values, budget, 0)))?,
        level: crate::grpc::evidence::canonical::severity(value.level),
    })
}

fn field_wire(field: &prompts::PromptField, budget: &mut Budget) -> Result<dokime::domain::model::prompts::wire::PromptField, Status> {
    budget.node(0)?;
    let validation = &field.validation;
    Ok(dokime::domain::model::prompts::wire::PromptField {
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
            prompts::PromptFieldKind::Text => dokime::domain::model::prompts::wire::PromptFieldKind::Text,
            prompts::PromptFieldKind::Float => dokime::domain::model::prompts::wire::PromptFieldKind::Float,
            prompts::PromptFieldKind::Int => dokime::domain::model::prompts::wire::PromptFieldKind::Int,
            prompts::PromptFieldKind::Choice => dokime::domain::model::prompts::wire::PromptFieldKind::Choice,
            prompts::PromptFieldKind::Boolean => dokime::domain::model::prompts::wire::PromptFieldKind::Boolean,
        },
        multiple: field.multiple,
        required: field.required,
        default_value: field.default.as_ref().map(|v| project_value(v, budget, 0)).transpose()?,
        choices: collect(field.choices.iter().map(|value| projection::text(value).map_err(failure)))?,
        validation: Some(dokime::domain::model::prompts::wire::PromptFieldValidation {
            min_length: validation.min_length.map(|v| v as u64),
            max_length: validation.max_length.map(|v| v as u64),
            pattern: validation.pattern.clone(),
            minimum: validation.minimum.as_ref().map(|v| project_value(v, budget, 0)).transpose()?,
            maximum: validation.maximum.as_ref().map(|v| project_value(v, budget, 0)).transpose()?,
            min_items: validation.min_items.map(|v| v as u64),
            max_items: validation.max_items.map(|v| v as u64),
        }),
    })
}

fn predicate(
    expression: &Expression, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::prompts::wire::AnswerPredicate, Status> {
    let mut nodes = Vec::new();
    push_predicate(expression, budget, depth, &mut nodes)?;
    Ok(dokime::domain::model::prompts::wire::AnswerPredicate { nodes })
}

fn push_predicate(
    expression: &Expression, budget: &mut Budget, depth: usize, nodes: &mut Vec<dokime::domain::model::prompts::wire::AnswerPredicateNode>,
) -> Result<u32, Status> {
    use dokime::domain::model::prompts::wire::AnswerPredicateKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len()).map_err(|_| Status::resource_exhausted("prompt predicate index overflow"))?;
    nodes.push(dokime::domain::model::prompts::wire::AnswerPredicateNode { kind: None });
    let kind = match &expression.kind {
        ExpressionKind::Literal { value } => Kind::Literal(project_value(value, budget, depth + 1)?),
        ExpressionKind::Selector { path } => Kind::Field(path.clone()),
        ExpressionKind::Unary { op: ExpressionUnaryOp::Not, value } => Kind::Not(push_predicate(value, budget, depth + 1, nodes)?),
        ExpressionKind::Binary { op, left, right } => Kind::Binary(dokime::domain::model::prompts::wire::AnswerBinaryPredicate {
            op: match op {
                ExpressionBinaryOp::And => dokime::domain::model::prompts::wire::AnswerOperator::And,
                ExpressionBinaryOp::Or => dokime::domain::model::prompts::wire::AnswerOperator::Or,
                ExpressionBinaryOp::Eq => dokime::domain::model::prompts::wire::AnswerOperator::Eq,
                ExpressionBinaryOp::Ne => dokime::domain::model::prompts::wire::AnswerOperator::Ne,
                ExpressionBinaryOp::Gt => dokime::domain::model::prompts::wire::AnswerOperator::Gt,
                ExpressionBinaryOp::Gte => dokime::domain::model::prompts::wire::AnswerOperator::Gte,
                ExpressionBinaryOp::Lt => dokime::domain::model::prompts::wire::AnswerOperator::Lt,
                ExpressionBinaryOp::Lte => dokime::domain::model::prompts::wire::AnswerOperator::Lte,
                ExpressionBinaryOp::Contains => dokime::domain::model::prompts::wire::AnswerOperator::Contains,
                ExpressionBinaryOp::StartsWith => dokime::domain::model::prompts::wire::AnswerOperator::StartsWith,
                ExpressionBinaryOp::EndsWith => dokime::domain::model::prompts::wire::AnswerOperator::EndsWith,
            },
            left: push_predicate(left, budget, depth + 1, nodes)?,
            right: push_predicate(right, budget, depth + 1, nodes)?,
        }),
        ExpressionKind::Call { .. } => {
            return Err(Status::internal("admitted prompt contains unsupported answer call"));
        }
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}

fn condition_wire(
    condition: &RuntimeCondition, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::prompts::wire::EvidenceCondition, Status> {
    let mut nodes = Vec::new();
    push_condition(condition, budget, depth, &mut nodes)?;
    Ok(dokime::domain::model::prompts::wire::EvidenceCondition { nodes })
}

fn push_condition(
    condition: &RuntimeCondition, budget: &mut Budget, depth: usize,
    nodes: &mut Vec<dokime::domain::model::prompts::wire::EvidenceConditionNode>,
) -> Result<u32, Status> {
    use dokime::domain::model::prompts::wire::EvidenceConditionKind as Kind;
    budget.node(depth)?;
    let index = u32::try_from(nodes.len()).map_err(|_| Status::resource_exhausted("prompt condition index overflow"))?;
    nodes.push(dokime::domain::model::prompts::wire::EvidenceConditionNode { kind: None });
    let kind = match condition {
        RuntimeCondition::Record { condition } => Kind::Record(dokime::domain::model::prompts::wire::RecordMatchCondition {
            record_type: condition.record_type.as_str().to_owned(),
            match_fields: Some(object(&condition.match_fields, budget, depth + 1)?),
        }),
        RuntimeCondition::Telemetry { condition } => {
            if condition.tolerance.is_some_and(|v| !v.is_finite()) {
                return Err(Status::internal("admitted prompt has nonfinite tolerance"));
            }
            Kind::Telemetry(dokime::domain::model::prompts::wire::TelemetryMatchCondition {
                stream: condition.stream.as_str().to_owned(),
                field: condition.field.clone(),
                op: match condition.operator {
                    TelemetryConditionOperator::Exists => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Exists,
                    TelemetryConditionOperator::Eq => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Eq,
                    TelemetryConditionOperator::Gt => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Gt,
                    TelemetryConditionOperator::Gte => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Gte,
                    TelemetryConditionOperator::Lt => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Lt,
                    TelemetryConditionOperator::Lte => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Lte,
                    TelemetryConditionOperator::Within => dokime::domain::model::prompts::wire::TelemetryMatchOperator::Within,
                },
                target: condition.target.as_ref().map(|v| project_value(v, budget, depth + 1)).transpose()?,
                tolerance: condition.tolerance,
            })
        }
        RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } => {
            let mut links = projection::slots(conditions.len()).map_err(failure)?;
            for child in conditions {
                links.push(push_condition(child, budget, depth + 1, nodes)?);
            }
            let group = dokime::domain::model::prompts::wire::EvidenceConditions { conditions: links };
            if matches!(condition, RuntimeCondition::All { .. }) { Kind::All(group) } else { Kind::Any(group) }
        }
    };
    nodes[index as usize].kind = Some(kind);
    Ok(index)
}

fn object(
    values: &BTreeMap<String, Value>, budget: &mut Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    budget.node(depth)?;
    canonical::object_at(values, budget, depth)
}

fn failure(error: dokime::harmos::Error) -> Status {
    match error {
        dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("canonical prompt construction exceeds its bound"),
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

fn metadata(source: &dokime::domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(source: &Value, budget: &mut Budget, depth: usize) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

/* === Tests === */

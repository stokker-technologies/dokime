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
use super::value::{self, Budget, projection, timestamp};
use dokime_app::capabilities::prompts;
use dokime_app::core::{Value, ValueType};
use dokime_app::definition::RuntimeCondition;
use dokime_app::expressions::{Expression, ExpressionBinaryOp, ExpressionKind, ExpressionUnaryOp};
use dokime_app::telemetry::TelemetryConditionOperator;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn identity(value: &prompts::PromptDefinition) -> v1::PromptDefinitionIdentity {
    v1::PromptDefinitionIdentity {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        title: value.title.clone(),
    }
}

pub(super) fn definition(
    value: &prompts::PromptDefinition,
) -> Result<v1::PromptDefinition, Status> {
    definition_heap(value, &mut Budget::new())?;
    definition_with_budget(value, &mut Budget::new())
}

pub(crate) fn instance(value: &prompts::PromptInstance) -> Result<v1::PromptInstance, Status> {
    instance_memory(value)?;
    let mut budget = Budget::new();
    Ok(v1::PromptInstance {
        prompt_id: value.prompt_id.as_str().to_owned(),
        definition: Some(definition_with_budget(&value.definition, &mut budget)?),
        kind: match value.kind {
            prompts::PromptKind::Confirmation => v1::PromptKind::Confirmation,
            prompts::PromptKind::Form => v1::PromptKind::Form,
            prompts::PromptKind::Choice => v1::PromptKind::Choice,
            prompts::PromptKind::Status => v1::PromptKind::Status,
            prompts::PromptKind::Verification => v1::PromptKind::Verification,
        } as i32,
        values: Some(object(&value.values, &mut budget, 0)?),
        scope: Some(projection::scope(&value.scope).map_err(failure)?),
        created_at: Some(timestamp(&value.created_at)?),
    })
}

pub(super) fn response(value: &prompts::PromptResponse) -> Result<v1::PromptResponse, Status> {
    Ok(v1::PromptResponse {
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
    let bytes = std::mem::size_of::<v1::PromptInstance>()
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
) -> Result<v1::PromptDefinition, Status> {
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
    Ok(v1::PromptDefinition {
        prompt_type: value.prompt_type.as_str().to_owned(),
        schema_version: value.schema_version,
        archetype: match value.archetype {
            prompts::PromptArchetype::ConfirmationPrompt => v1::PromptArchetype::Confirmation,
            prompts::PromptArchetype::FormPrompt => v1::PromptArchetype::Form,
            prompts::PromptArchetype::ChoicePrompt => v1::PromptArchetype::Choice,
            prompts::PromptArchetype::StatusPrompt => v1::PromptArchetype::Status,
        } as i32,
        title: value.title.clone(),
        message: value.message.clone(),
        fields: collect(value.fields.iter().map(|field| field_wire(field, budget)))?,
        actions: collect(value.actions.iter().map(|action| {
            budget.node(0)?;
            value::text(&action.id)?;
            value::text(&action.label)?;
            Ok(v1::PromptAction {
                id: action.id.clone(),
                label: action.label.clone(),
                emphasis: match action.emphasis {
                    prompts::PromptActionEmphasis::Primary => v1::PromptActionEmphasis::Primary,
                    prompts::PromptActionEmphasis::Secondary => v1::PromptActionEmphasis::Secondary,
                    prompts::PromptActionEmphasis::Danger => v1::PromptActionEmphasis::Danger,
                } as i32,
                requires_condition: action.requires_condition,
            })
        }))?,
        rules: collect(value.rules.iter().map(|rule| {
            value::text(&rule.message)?;
            Ok(v1::AnswerRule {
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
    })
}

fn field_wire(
    field: &prompts::PromptField,
    budget: &mut Budget,
) -> Result<v1::PromptField, Status> {
    budget.node(0)?;
    value::collection(field.choices.len())?;
    for choice in &field.choices {
        value::text(choice)?;
    }
    let validation = &field.validation;
    if let Some(pattern) = &validation.pattern {
        value::text(pattern)?;
    }
    Ok(v1::PromptField {
        metadata: Some(metadata(&field.metadata)?),
        value_type: match field.value_type {
            ValueType::Boolean => v1::ValueType::Boolean,
            ValueType::Integer => v1::ValueType::Integer,
            ValueType::Number => v1::ValueType::Number,
            ValueType::Text => v1::ValueType::Text,
            ValueType::Datetime => v1::ValueType::Datetime,
            ValueType::File => v1::ValueType::File,
            ValueType::Object => v1::ValueType::Object,
            ValueType::Array => v1::ValueType::Array,
        } as i32,
        kind: match field.kind {
            prompts::PromptFieldKind::Text => v1::PromptFieldKind::Text,
            prompts::PromptFieldKind::Float => v1::PromptFieldKind::Float,
            prompts::PromptFieldKind::Int => v1::PromptFieldKind::Int,
            prompts::PromptFieldKind::Choice => v1::PromptFieldKind::Choice,
            prompts::PromptFieldKind::Boolean => v1::PromptFieldKind::Boolean,
        } as i32,
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
        validation: Some(v1::PromptFieldValidation {
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
) -> Result<v1::AnswerPredicate, Status> {
    use v1::answer_predicate::Kind;
    budget.node(depth)?;
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
        } => Kind::Not(Box::new(predicate(value, budget, depth + 1)?)),
        ExpressionKind::Binary { op, left, right } => {
            Kind::Binary(Box::new(v1::AnswerBinaryPredicate {
                op: match op {
                    ExpressionBinaryOp::And => v1::AnswerOperator::And,
                    ExpressionBinaryOp::Or => v1::AnswerOperator::Or,
                    ExpressionBinaryOp::Eq => v1::AnswerOperator::Eq,
                    ExpressionBinaryOp::Ne => v1::AnswerOperator::Ne,
                    ExpressionBinaryOp::Gt => v1::AnswerOperator::Gt,
                    ExpressionBinaryOp::Gte => v1::AnswerOperator::Gte,
                    ExpressionBinaryOp::Lt => v1::AnswerOperator::Lt,
                    ExpressionBinaryOp::Lte => v1::AnswerOperator::Lte,
                    ExpressionBinaryOp::Contains => v1::AnswerOperator::Contains,
                    ExpressionBinaryOp::StartsWith => v1::AnswerOperator::StartsWith,
                    ExpressionBinaryOp::EndsWith => v1::AnswerOperator::EndsWith,
                } as i32,
                left: Some(Box::new(predicate(left, budget, depth + 1)?)),
                right: Some(Box::new(predicate(right, budget, depth + 1)?)),
            }))
        }
        ExpressionKind::Call { .. } => {
            return Err(Status::internal(
                "admitted prompt contains unsupported answer call",
            ));
        }
    };
    Ok(v1::AnswerPredicate { kind: Some(kind) })
}

fn condition_wire(
    condition: &RuntimeCondition,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::EvidenceCondition, Status> {
    use v1::evidence_condition::Kind;
    budget.node(depth)?;
    Ok(v1::EvidenceCondition {
        kind: Some(match condition {
            RuntimeCondition::Record { condition } => Kind::Record(v1::RecordMatchCondition {
                record_type: condition.record_type.as_str().to_owned(),
                match_fields: Some(object(&condition.match_fields, budget, depth + 1)?),
            }),
            RuntimeCondition::Telemetry { condition } => {
                value::text(&condition.field)?;
                if condition.tolerance.is_some_and(|v| !v.is_finite()) {
                    return Err(Status::internal("admitted prompt has nonfinite tolerance"));
                }
                Kind::Telemetry(v1::TelemetryMatchCondition {
                    stream: condition.stream.as_str().to_owned(),
                    field: condition.field.clone(),
                    op: match condition.operator {
                        TelemetryConditionOperator::Exists => v1::TelemetryMatchOperator::Exists,
                        TelemetryConditionOperator::Eq => v1::TelemetryMatchOperator::Eq,
                        TelemetryConditionOperator::Gt => v1::TelemetryMatchOperator::Gt,
                        TelemetryConditionOperator::Gte => v1::TelemetryMatchOperator::Gte,
                        TelemetryConditionOperator::Lt => v1::TelemetryMatchOperator::Lt,
                        TelemetryConditionOperator::Lte => v1::TelemetryMatchOperator::Lte,
                        TelemetryConditionOperator::Within => v1::TelemetryMatchOperator::Within,
                    } as i32,
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
                let values = v1::EvidenceConditions {
                    conditions: collect(
                        conditions
                            .iter()
                            .map(|v| condition_wire(v, budget, depth + 1)),
                    )?,
                };
                if matches!(condition, RuntimeCondition::All { .. }) {
                    Kind::All(values)
                } else {
                    Kind::Any(values)
                }
            }
        }),
    })
}

fn object(
    values: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<v1::ValueObject, Status> {
    budget.node(depth)?;
    projection::object_count(values, &mut Budget::new(), depth).map_err(failure)?;
    projection::object(values, budget, depth).map_err(failure)
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

fn metadata(source: &dokime_app::core::Metadata) -> Result<v1::Metadata, Status> {
    projection::metadata_fields_count(source).map_err(failure)?;
    projection::metadata(source).map_err(failure)
}

fn project_value(source: &Value, budget: &mut Budget, depth: usize) -> Result<v1::Value, Status> {
    projection::count(source, &mut Budget::new(), depth).map_err(failure)?;
    projection::project(source, budget, depth).map_err(failure)
}

fn scope_heap(source: &dokime_app::core::RecordScope) -> Result<usize, Status> {
    Ok(bounded(projection::scope_memory(source))? - std::mem::size_of::<v1::EvidenceScope>())
}

fn object_heap(
    source: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<usize, Status> {
    budget.node(depth)?;
    Ok(projection::object_count(source, budget, depth)
        .map_err(failure)?
        .memory
        - std::mem::size_of::<v1::ValueObject>())
}

fn value_heap(source: &Value, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    Ok(projection::count(source, budget, depth)
        .map_err(failure)?
        .memory
        - std::mem::size_of::<v1::Value>())
}

fn slots<T>(length: usize) -> usize {
    length.saturating_mul(std::mem::size_of::<T>())
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
        .saturating_add(slots::<v1::PromptField>(source.fields.len()))
        .saturating_add(slots::<v1::PromptAction>(source.actions.len()))
        .saturating_add(slots::<v1::AnswerRule>(source.rules.len()))
        .saturating_add(slots::<v1::ValueObject>(source.examples.len()));
    for field in &source.fields {
        budget.node(0)?;
        value::collection(field.choices.len())?;
        bytes = bytes
            .saturating_add(
                projection::metadata_fields_count(&field.metadata)
                    .map_err(failure)?
                    .memory
                    - std::mem::size_of::<v1::Metadata>(),
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
        } => std::mem::size_of::<v1::AnswerPredicate>().saturating_add(predicate_heap(
            value,
            budget,
            depth + 1,
        )?),
        ExpressionKind::Binary { left, right, .. } => {
            std::mem::size_of::<v1::AnswerBinaryPredicate>()
                .saturating_add(slots::<v1::AnswerPredicate>(2))
                .saturating_add(predicate_heap(left, budget, depth + 1)?)
                .saturating_add(predicate_heap(right, budget, depth + 1)?)
        }
        ExpressionKind::Call { .. } => {
            return Err(Status::internal(
                "admitted prompt contains unsupported answer call",
            ));
        }
    };
    bounded(bytes)
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
            let mut bytes = slots::<v1::EvidenceCondition>(conditions.len());
            for condition in conditions {
                bytes = bytes.saturating_add(condition_heap(condition, budget, depth + 1)?);
            }
            bytes
        }
    };
    bounded(bytes)
}

/* === Tests === */

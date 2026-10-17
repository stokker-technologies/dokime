/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Structural Prompt Definition, Field And Rule Validation.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use crate::model::diagnostics::ValidationIssue;
use crate::model::expressions::ExpressionContext;
use crate::model::prompts::PromptArchetype;
use crate::model::prompts::PromptDefinition;
use crate::model::prompts::PromptField;
use crate::model::prompts::PromptFieldKind;
use crate::model::prompts::validate_rules;
use crate::model::values::Value;
use crate::model::values::ValueType;
use crate::model::values::numeric_cmp;
use crate::model::values::value_matches_type;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn validate_fields(fields: &[PromptField], values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for name in values.keys() {
        if !fields.iter().any(|field| field.metadata.identifier.as_str() == name) {
            issue(&mut issues, name, "field is not declared by this prompt");
        }
    }
    for field in fields {
        let name = field.metadata.identifier.as_str();
        let Some(value) = values.get(name) else {
            if field.required {
                issue(&mut issues, name, "required prompt field is missing");
            }
            continue;
        };
        if matches!(value, Value::Null) && !field.required {
            continue;
        }
        if field.multiple {
            let Value::Array(items) = value else {
                issue(&mut issues, name, "multiple values must be an array");
                continue;
            };
            let minimum = field.validation.min_items.unwrap_or(usize::from(field.required));
            if items.len() < minimum || field.validation.max_items.is_some_and(|max| items.len() > max) {
                issue(&mut issues, name, "selection count is outside the permitted bounds");
            }
            for (index, item) in items.iter().enumerate() {
                let path = format!("{name}.{index}");
                validate_item(field, item, &path, &mut issues);
                if field.kind == PromptFieldKind::Choice && items[..index].contains(item) {
                    issue(&mut issues, &path, "a choice may only be selected once");
                }
            }
        } else {
            validate_item(field, value, name, &mut issues);
        }
    }
    issues
}

pub fn validate_definition(definition: &PromptDefinition) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut actions = BTreeSet::new();
    for action in &definition.actions {
        if action.requires_condition && definition.condition.is_none() {
            declaration_issue(&mut issues, "condition", "a gated action requires a condition");
        }
        if action.id.trim().is_empty() || !actions.insert(&action.id) {
            declaration_issue(&mut issues, "actions", "action identifiers must be nonempty and unique");
        }
    }
    if definition.archetype != PromptArchetype::StatusPrompt && actions.is_empty() {
        declaration_issue(&mut issues, "actions", "a question must declare an action");
    }
    let mut names = BTreeSet::new();
    for field in &definition.fields {
        let path = format!("fields.{}", field.metadata.identifier);
        let mut invalid = |message: &str| declaration_issue(&mut issues, &path, message);
        if !names.insert(&field.metadata.identifier) {
            invalid("field identifiers must be unique");
        }
        let expected = match field.kind {
            PromptFieldKind::Text | PromptFieldKind::Choice => ValueType::Text,
            PromptFieldKind::Float => ValueType::Number,
            PromptFieldKind::Int => ValueType::Integer,
            PromptFieldKind::Boolean => ValueType::Boolean,
        };
        if field.value_type != expected {
            invalid("field kind and value type disagree");
        }
        let rules = &field.validation;
        if rules.min_length.zip(rules.max_length).is_some_and(|(min, max)| min > max) {
            invalid("minimum length exceeds maximum length");
        }
        if rules.min_items.zip(rules.max_items).is_some_and(|(min, max)| min > max) {
            invalid("minimum selections exceeds maximum selections");
        }
        if !field.multiple && (rules.min_items.is_some() || rules.max_items.is_some()) {
            invalid("selection bounds require a multiple field");
        }
        if field.value_type != ValueType::Text && (rules.pattern.is_some() || rules.min_length.is_some() || rules.max_length.is_some()) {
            invalid("text rules require a text field");
        }
        for bound in [&rules.minimum, &rules.maximum].into_iter().flatten() {
            if !matches!(field.value_type, ValueType::Integer | ValueType::Number) || !value_matches_type(bound, &field.value_type) {
                invalid("numeric bounds must match the field type and be finite");
            }
        }
        if let (Some(min), Some(max)) = (&rules.minimum, &rules.maximum)
            && numeric_cmp(min, max).is_none_or(|order| order.is_gt())
        {
            invalid("minimum exceeds maximum");
        }
        if field.kind == PromptFieldKind::Choice {
            let choices: BTreeSet<_> = field.choices.iter().collect();
            if choices.is_empty() || choices.len() != field.choices.len() {
                invalid("choices must be nonempty and unique");
            }
            if rules.min_items.is_some_and(|minimum| minimum > choices.len()) {
                invalid("minimum selections exceeds available choices");
            }
        } else if !field.choices.is_empty() {
            invalid("only choice fields can declare choices");
        }
        if field.required && rules.max_items == Some(0) {
            invalid("a required selection must allow at least one value");
        }
    }
    issues.extend(validate_rules(definition));
    issues
}

pub fn validate_answers(definition: &PromptDefinition, values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    let mut issues = validate_fields(&definition.fields, values);
    if !issues.is_empty() {
        return issues;
    }
    issues.extend(validate_answer_rules(definition, values));
    issues
}

pub fn validate_answer_rules(definition: &PromptDefinition, values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    // Bound evaluation even for definitions supplied directly outside installation.
    issues.extend(validate_rules(definition));
    if !issues.is_empty() {
        return issues;
    }
    let context = ExpressionContext::new(values.clone());
    for (index, rule) in definition.rules.iter().enumerate() {
        if !matches!(rule.predicate.evaluate(&context), Ok(true)) {
            issues.push(rule_issue(index, &rule.message));
        }
    }
    issues
}

/// Collects the one definition for each prompt identity, ordered by prompt identity.
pub fn collect_definitions<'a>(
    source: &str, definitions: impl IntoIterator<Item = &'a PromptDefinition>,
) -> Result<Vec<&'a PromptDefinition>, String> {
    let mut declarations = BTreeMap::new();
    for definition in definitions {
        let id = definition.prompt_type.as_str();
        if let Some(existing) = declarations.get(id) {
            if *existing != definition {
                return Err(format!("conflicting definitions for prompt {id} in {source}"));
            }
            continue;
        }
        declarations.insert(id, definition);
    }
    Ok(declarations.into_values().collect())
}

/* === Private Functions === */
fn validate_item(field: &PromptField, value: &Value, name: &str, issues: &mut Vec<ValidationIssue>) {
    // The field's value type describes each item, not the collection envelope.
    if !value_matches_type(value, &field.value_type) {
        issue(issues, name, "prompt field has an invalid type or non-finite number");
        return;
    }
    if let Value::Text(text) = value {
        let length = text.chars().count();
        if field.required && text.trim().is_empty() {
            issue(issues, name, "required text cannot be empty");
        }
        if field.validation.min_length.is_some_and(|min| length < min) || field.validation.max_length.is_some_and(|max| length > max) {
            issue(issues, name, "text length is outside the permitted bounds");
        }
        if field.kind == PromptFieldKind::Choice && !field.choices.contains(text) {
            issue(issues, name, "value is not an offered choice");
        }
    }
    if let Some(minimum) = &field.validation.minimum
        && numeric_cmp(value, minimum).is_none_or(|order| order.is_lt())
    {
        issue(issues, name, "value is below the permitted minimum");
    }
    if let Some(maximum) = &field.validation.maximum
        && numeric_cmp(value, maximum).is_none_or(|order| order.is_gt())
    {
        issue(issues, name, "value exceeds the permitted maximum");
    }
}

fn issue(issues: &mut Vec<ValidationIssue>, name: &str, message: &str) {
    issues.push(ValidationIssue { path: format!("values.{name}"), message: message.to_owned(), expected: None });
}

fn declaration_issue(issues: &mut Vec<ValidationIssue>, path: &str, message: &str) {
    issues.push(ValidationIssue { path: path.into(), message: message.into(), expected: None });
}

fn rule_issue(index: usize, message: &str) -> ValidationIssue {
    ValidationIssue { path: format!("rules.{index}"), message: message.into(), expected: None }
}

/* === Tests === */

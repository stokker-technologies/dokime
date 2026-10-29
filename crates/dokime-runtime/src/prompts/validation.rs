/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompt definitions and answers checked against their declared rules: text
// patterns compile with `regex`, so the checks live with the runtime.

use std::collections::BTreeMap;

use dokime_domain::model::diagnostics::ValidationIssue;
use dokime_domain::model::prompts::{self, PromptDefinition, PromptField};
use dokime_domain::model::values::{Value, ValueType};

pub fn validate_fields(fields: &[PromptField], values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    let mut issues = prompts::validate_fields(fields, values);
    for field in fields {
        if field.value_type != ValueType::Text {
            continue;
        }
        let Some(value) = values.get(field.metadata.identifier.as_str()) else {
            continue;
        };
        let Some(pattern) = &field.validation.pattern else {
            continue;
        };
        let name = field.metadata.identifier.as_str();
        if field.multiple {
            if let Value::Array(items) = value {
                for (index, value) in items.iter().enumerate() {
                    validate_pattern(pattern, value, &format!("{name}.{index}"), &mut issues);
                }
            }
        } else {
            validate_pattern(pattern, value, name, &mut issues);
        }
    }
    issues
}

pub fn validate_definition(definition: &PromptDefinition) -> Vec<ValidationIssue> {
    let mut issues = prompts::validate_definition(definition);
    for field in &definition.fields {
        if field.validation.pattern.as_ref().is_some_and(|pattern| regex::Regex::new(pattern).is_err()) {
            issues.push(ValidationIssue {
                path: format!("fields.{}", field.metadata.identifier),
                message: "invalid text pattern".to_owned(),
                expected: None,
            });
        }
    }
    issues
}

pub fn validate_answers(definition: &PromptDefinition, values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    let issues = validate_fields(&definition.fields, values);
    if !issues.is_empty() {
        return issues;
    }
    prompts::validate_answer_rules(definition, values)
}

fn validate_pattern(pattern: &str, value: &Value, name: &str, issues: &mut Vec<ValidationIssue>) {
    let Value::Text(text) = value else {
        return;
    };
    let message = match regex::Regex::new(pattern) {
        Ok(pattern) if pattern.is_match(text) => return,
        Ok(_) => "text does not match the required pattern",
        Err(_) => "prompt declares an invalid pattern",
    };
    issues.push(ValidationIssue {
        path: format!("values.{name}"),
        message: message.to_owned(),
        expected: None,
    });
}

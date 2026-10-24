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

// Local Imports
use super::authoring::{action, definition, field, validate_response};
use super::{PromptActionEmphasis, PromptArchetype, PromptDefinition, PromptField, PromptFieldKind, PromptResponse};
use crate::values::FieldValue;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

pub trait DeclaredPrompt: Sized + Send + 'static {
    type Answer: Send;

    const PROMPT_TYPE: &'static str;

    fn definition() -> PromptDefinition;

    fn decode(definition: &PromptDefinition, response: PromptResponse) -> Result<Self::Answer, crate::Error>;
}

pub trait PromptFieldValue: FieldValue {
    const KIND: PromptFieldKind;

    fn choices() -> Vec<String> {
        Vec::new()
    }
}

/* === Implementations === */

impl PromptFieldValue for String {
    const KIND: PromptFieldKind = PromptFieldKind::Text;
}

impl PromptFieldValue for bool {
    const KIND: PromptFieldKind = PromptFieldKind::Boolean;
}

impl PromptFieldValue for i64 {
    const KIND: PromptFieldKind = PromptFieldKind::Int;
}

impl PromptFieldValue for i32 {
    const KIND: PromptFieldKind = PromptFieldKind::Int;
}

impl PromptFieldValue for u64 {
    const KIND: PromptFieldKind = PromptFieldKind::Int;
}

impl PromptFieldValue for u32 {
    const KIND: PromptFieldKind = PromptFieldKind::Int;
}

impl PromptFieldValue for u16 {
    const KIND: PromptFieldKind = PromptFieldKind::Int;
}

impl PromptFieldValue for f64 {
    const KIND: PromptFieldKind = PromptFieldKind::Float;
}

impl PromptFieldValue for f32 {
    const KIND: PromptFieldKind = PromptFieldKind::Float;
}

/* === Public Functions === */

pub fn declared_form(id: &str, label: impl Into<String>) -> PromptDefinition {
    definition(id, label.into(), PromptArchetype::FormPrompt)
}

pub fn declared_confirmation(id: &str, label: impl Into<String>) -> PromptDefinition {
    let mut declared = definition(id, label.into(), PromptArchetype::ConfirmationPrompt);
    declared.actions = vec![
        action("yes", "Yes", PromptActionEmphasis::Primary),
        action("no", "No", PromptActionEmphasis::Secondary),
        action("cancel", "Cancel", PromptActionEmphasis::Secondary),
    ];

    declared
}

pub fn declared_field<T: PromptFieldValue>(id: &str, label: impl Into<String>, required: bool, multiple: bool) -> PromptField {
    let mut declared = field(id, label.into(), T::KIND, T::VALUE_TYPE);
    declared.required = required;
    declared.multiple = multiple;
    declared.choices = T::choices();

    declared
}

pub fn declared_default<T: FieldValue>(value: impl Into<T>) -> Value {
    value.into().into_value()
}

pub fn answered<T: FieldValue>(values: &BTreeMap<String, Value>, name: &str) -> Result<T, crate::Error> {
    T::from_value(values.get(name)).map_err(|error| {
        crate::Error::PromptRefused(format!(
            "field `{name}` is not the {} it was declared as: {}",
            describe(error.declared),
            error.found.unwrap_or_else(|| "nothing was answered".to_owned())
        ))
    })
}

pub fn form_answer(definition: &PromptDefinition, response: &PromptResponse) -> Result<(), crate::Error> {
    validate_response(definition, response)
}

pub fn confirmation_answer(definition: &PromptDefinition, response: &PromptResponse) -> Result<bool, crate::Error> {
    validate_response(definition, response)?;
    match response.action.as_str() {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(crate::Error::PromptCancelled),
    }
}

pub fn declared_metadata(id: &str, label: impl Into<String>, description: Option<String>) -> Metadata {
    Metadata::new(Identifier::new(id).expect("a prompt field identifier is valid"), label.into(), description)
}

/* === Private Functions === */

fn describe(declared: ValueType) -> &'static str {
    match declared {
        ValueType::Text => "text",
        ValueType::Boolean => "boolean",
        ValueType::Integer => "integer",
        ValueType::Number => "number",
        _ => "value",
    }
}

/* === Tests === */

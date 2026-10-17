/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod private {
    pub trait Number {}
    impl Number for i64 {}
    impl Number for f64 {}
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::marker::PhantomData;
// Third Party Imports
use serde::de::DeserializeOwned;
// Local Imports
use super::*;
use crate::records::RecordSeverity;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct FormPrompt<T> {
    definition: PromptDefinition,
    response: PhantomData<fn() -> T>,
}

pub struct TextPrompt {
    form: FormPrompt<BTreeMap<String, String>>,
}

pub struct NumberPrompt<T = f64> {
    form: FormPrompt<BTreeMap<String, T>>,
}

pub struct ConfirmationPrompt {
    definition: PromptDefinition,
}

pub struct ChoicePrompt<T> {
    form: FormPrompt<BTreeMap<String, String>>,
    choices: BTreeMap<String, T>,
}

pub struct MultipleChoicePrompt<T> {
    form: FormPrompt<BTreeMap<String, Vec<String>>>,
    choices: BTreeMap<String, T>,
}

/* === Enums === */

/* === Traits === */
pub trait Prompt: Send {
    type Response;

    fn definition(&self) -> PromptDefinition;
    fn decode(&self, response: PromptResponse) -> Result<Self::Response, crate::Error>;

    fn request(&self) -> PromptRequest {
        let definition = self.definition();
        PromptRequest {
            prompt_id: crate::model::identity::new_id::<PromptId>(),
            prompt_type: definition.prompt_type,
            values: definition
                .fields
                .into_iter()
                .filter_map(|field| field.default.map(|value| (field.metadata.identifier.to_string(), value)))
                .collect(),
            scope: RecordScope::default(),
        }
    }
}

pub trait PromptNumber: private::Number + DeserializeOwned + Into<Value> + Copy {
    fn field_kind() -> PromptFieldKind;
    fn value_type() -> ValueType;
}

/* === Implementations === */
impl<T> FormPrompt<T> {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        Self { definition: definition(id, label.into(), PromptArchetype::FormPrompt), response: PhantomData }
    }

    pub fn field(mut self, field: PromptField) -> Self {
        self.definition.fields.push(field);
        self
    }

    pub fn rule(mut self, predicate: crate::expressions::BooleanExpression, message: impl Into<String>) -> Self {
        self.definition.rules.push(Rule { predicate, message: message.into() });
        self
    }

    pub fn default(mut self, name: &str, value: impl Into<Value>) -> Self {
        let field = self
            .definition
            .fields
            .iter_mut()
            .find(|field| field.metadata.identifier.as_str() == name)
            .expect("a form default names a declared field");
        field.default = Some(value.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.message = Some(description.into());
        self
    }

    pub fn submit(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.definition.actions[0].id = id.into();
        self.definition.actions[0].label = label.into();
        self
    }

    pub fn labels(mut self, submit: impl Into<String>, cancel: impl Into<String>) -> Self {
        self.definition.actions[0].label = submit.into();
        self.definition.actions[1].label = cancel.into();
        self
    }

    pub fn level(mut self, level: RecordSeverity) -> Self {
        self.definition.level = level;
        self
    }
}

impl<T: DeserializeOwned> Prompt for FormPrompt<T> {
    type Response = T;

    fn definition(&self) -> PromptDefinition {
        self.definition.clone()
    }

    fn decode(&self, response: PromptResponse) -> Result<T, crate::Error> {
        validate_response(&self.definition, &response)?;
        let value = dokime_domain::model::values::Value::Object(response.values).to_plain_json().map_err(refused)?;
        let answer = serde_json::from_value(value).map_err(refused)?;
        Ok(answer)
    }
}

impl TextPrompt {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        let label = label.into();
        Self { form: FormPrompt::new(id, label.clone()).field(field(id, label, PromptFieldKind::Text, ValueType::Text)) }
    }
    pub fn length(mut self, minimum: usize, maximum: usize) -> Self {
        self.form.definition.fields[0].validation.min_length = Some(minimum);
        self.form.definition.fields[0].validation.max_length = Some(maximum);
        self
    }
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.form.definition.fields[0].validation.pattern = Some(pattern.into());
        self
    }
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.form.definition.fields[0].default = Some(Value::Text(value.into()));
        self
    }
    pub fn field(self) -> PromptField {
        self.form.definition.fields.into_iter().next().expect("a text prompt owns one field")
    }
}

impl Prompt for TextPrompt {
    type Response = String;
    fn definition(&self) -> PromptDefinition {
        self.form.definition()
    }
    fn decode(&self, response: PromptResponse) -> Result<String, crate::Error> {
        let mut values = self.form.decode(response)?;
        values
            .remove(self.form.definition.prompt_type.as_str())
            .ok_or_else(|| crate::Error::PromptRefused("text response is missing".to_owned()))
    }
}

impl PromptNumber for i64 {
    fn field_kind() -> PromptFieldKind {
        PromptFieldKind::Int
    }
    fn value_type() -> ValueType {
        ValueType::Integer
    }
}

impl PromptNumber for f64 {
    fn field_kind() -> PromptFieldKind {
        PromptFieldKind::Float
    }
    fn value_type() -> ValueType {
        ValueType::Number
    }
}

impl<T: PromptNumber> NumberPrompt<T> {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        let label = label.into();
        Self { form: FormPrompt::new(id, label.clone()).field(field(id, label, T::field_kind(), T::value_type())) }
    }
    pub fn bounds(mut self, minimum: T, maximum: T) -> Self {
        self.form.definition.fields[0].validation.minimum = Some(minimum.into());
        self.form.definition.fields[0].validation.maximum = Some(maximum.into());
        self
    }
    pub fn default(mut self, value: T) -> Self {
        self.form.definition.fields[0].default = Some(value.into());
        self
    }
    pub fn field(self) -> PromptField {
        self.form.definition.fields.into_iter().next().expect("a numeric prompt owns one field")
    }
}

impl<T: PromptNumber> Prompt for NumberPrompt<T> {
    type Response = T;
    fn definition(&self) -> PromptDefinition {
        self.form.definition()
    }
    fn decode(&self, response: PromptResponse) -> Result<T, crate::Error> {
        let mut values = self.form.decode(response)?;
        values
            .remove(self.form.definition.prompt_type.as_str())
            .ok_or_else(|| crate::Error::PromptRefused("numeric response is missing".to_owned()))
    }
}

impl ConfirmationPrompt {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        let mut definition = definition(id, label.into(), PromptArchetype::ConfirmationPrompt);
        definition.actions = vec![
            action("yes", "Yes", PromptActionEmphasis::Primary),
            action("no", "No", PromptActionEmphasis::Secondary),
            action("cancel", "Cancel", PromptActionEmphasis::Secondary),
        ];
        Self { definition }
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.message = Some(description.into());
        self
    }
    pub fn level(mut self, level: RecordSeverity) -> Self {
        self.definition.level = level;
        self
    }
    pub fn field(self) -> PromptField {
        field(self.definition.prompt_type.as_str(), self.definition.title, PromptFieldKind::Boolean, ValueType::Boolean)
    }
}

impl Prompt for ConfirmationPrompt {
    type Response = bool;
    fn definition(&self) -> PromptDefinition {
        self.definition.clone()
    }
    fn decode(&self, response: PromptResponse) -> Result<bool, crate::Error> {
        validate_response(&self.definition, &response)?;
        match response.action.as_str() {
            "yes" => Ok(true),
            "no" => Ok(false),
            _ => Err(crate::Error::PromptCancelled),
        }
    }
}

impl<T> ChoicePrompt<T> {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        let label = label.into();
        let mut form = FormPrompt::new(id, label.clone()).field(field(id, label, PromptFieldKind::Choice, ValueType::Text));
        form.definition.archetype = PromptArchetype::ChoicePrompt;
        Self { form, choices: BTreeMap::new() }
    }
    pub fn choice(mut self, key: impl Into<String>, value: T) -> Self {
        let key = key.into();
        self.form.definition.fields[0].choices.push(key.clone());
        self.choices.insert(key, value);
        self
    }
}

impl<T: Clone + Send> Prompt for ChoicePrompt<T> {
    type Response = T;
    fn definition(&self) -> PromptDefinition {
        self.form.definition()
    }
    fn decode(&self, response: PromptResponse) -> Result<T, crate::Error> {
        let values = self.form.decode(response)?;
        values
            .get(self.form.definition.prompt_type.as_str())
            .and_then(|key| self.choices.get(key))
            .cloned()
            .ok_or_else(|| crate::Error::PromptRefused("choice response is missing".to_owned()))
    }
}

impl<T> MultipleChoicePrompt<T> {
    pub fn new(id: &str, label: impl Into<String>) -> Self {
        let label = label.into();
        let mut item = field(id, label.clone(), PromptFieldKind::Choice, ValueType::Text);
        item.multiple = true;
        let mut form = FormPrompt::new(id, label).field(item);
        form.definition.archetype = PromptArchetype::ChoicePrompt;
        Self { form, choices: BTreeMap::new() }
    }
    pub fn choice(mut self, key: impl Into<String>, value: T) -> Self {
        let key = key.into();
        self.form.definition.fields[0].choices.push(key.clone());
        self.choices.insert(key, value);
        self
    }
    pub fn selections(mut self, minimum: usize, maximum: usize) -> Self {
        self.form.definition.fields[0].validation.min_items = Some(minimum);
        self.form.definition.fields[0].validation.max_items = Some(maximum);
        self
    }
}

impl<T: Clone + Send> Prompt for MultipleChoicePrompt<T> {
    type Response = Vec<T>;
    fn definition(&self) -> PromptDefinition {
        self.form.definition()
    }
    fn decode(&self, response: PromptResponse) -> Result<Vec<T>, crate::Error> {
        let values = self.form.decode(response)?;
        let keys = values
            .get(self.form.definition.prompt_type.as_str())
            .ok_or_else(|| crate::Error::PromptRefused("selection response is missing".to_owned()))?;
        keys.iter()
            .map(|key| self.choices.get(key).cloned().ok_or_else(|| crate::Error::PromptRefused("unknown choice".to_owned())))
            .collect()
    }
}

/* === Public Functions === */
pub(super) fn definition(id: &str, label: String, archetype: PromptArchetype) -> PromptDefinition {
    PromptDefinition {
        prompt_type: Identifier::new(id).expect("a prompt identifier is valid"),
        schema_version: 1,
        archetype,
        title: label,
        message: None,
        fields: Vec::new(),
        actions: vec![
            action("submit", "Submit", PromptActionEmphasis::Primary),
            action("cancel", "Cancel", PromptActionEmphasis::Secondary),
        ],
        rules: Vec::new(),
        condition: None,
        owner: None,
        template_identifier: None,
        examples: Vec::new(),
        level: RecordSeverity::Info,
    }
}

pub(super) fn field(id: &str, label: String, kind: PromptFieldKind, value_type: ValueType) -> PromptField {
    PromptField {
        metadata: Metadata::new(Identifier::new(id).expect("a prompt field identifier is valid"), label, None),
        value_type,
        kind,
        multiple: false,
        required: true,
        default: None,
        choices: Vec::new(),
        validation: FieldValidation::default(),
    }
}

pub(super) fn action(id: &str, label: &str, emphasis: PromptActionEmphasis) -> PromptAction {
    PromptAction { id: id.to_owned(), label: label.to_owned(), emphasis, requires_condition: false }
}

pub(super) fn validate_response(definition: &PromptDefinition, response: &PromptResponse) -> Result<(), crate::Error> {
    if !response.accepted || response.action == "cancel" {
        return Err(crate::Error::PromptCancelled);
    }
    if !definition.actions.iter().any(|action| action.id == response.action) {
        return Err(crate::Error::PromptRefused("response action is not declared".to_owned()));
    }
    let issues = validate_answers(definition, &response.values);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(crate::Error::PromptRefused(
            issues.into_iter().map(|issue| format!("{}: {}", issue.path, issue.message)).collect::<Vec<_>>().join("; "),
        ))
    }
}

/* === Private Functions === */
fn refused(error: impl std::fmt::Display) -> crate::Error {
    crate::Error::PromptRefused(error.to_string())
}

/* === Tests === */

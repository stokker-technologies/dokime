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

// Third Party Imports

// Local Imports
use super::{Map, Value, conditions, expressions};
use crate::prompts::{
    FieldValidation, PromptAction, PromptActionEmphasis, PromptArchetype, PromptDefinition,
    PromptField, PromptFieldKind, Rule as PromptRule,
};
use crate::records::RecordSeverity;
use crate::storage::objects::metadata;
use crate::storage::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prompt {
    #[serde(default)]
    prompt_type: String,
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    archetype: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    fields: Vec<Field>,
    #[serde(default)]
    actions: Vec<Action>,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default)]
    condition: Option<conditions::Condition>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    template_identifier: Option<String>,
    #[serde(default)]
    examples: Vec<Map>,
    // Empty reads as Info, the level a prompt has unless its author raised it.
    #[serde(default)]
    level: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Field {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    value_type: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    multiple: bool,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    default: Option<Value>,
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    validation: Validation,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Action {
    #[serde(default)]
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    emphasis: String,
    #[serde(default)]
    requires_condition: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Rule {
    #[serde(default)]
    predicate: expressions::Boolean,
    #[serde(default)]
    message: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Validation {
    #[serde(default)]
    min_length: Option<u64>,
    #[serde(default)]
    max_length: Option<u64>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    minimum: Option<Value>,
    #[serde(default)]
    maximum: Option<Value>,
    #[serde(default)]
    min_items: Option<u64>,
    #[serde(default)]
    max_items: Option<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Prompt {
    pub fn encoding_memory(value: &PromptDefinition) -> Result<usize> {
        let condition = value
            .condition
            .as_ref()
            .map(conditions::Condition::encoding_memory)
            .transpose()?
            .unwrap_or(0);
        if condition == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            value.prompt_type.as_str().len(),
            archetype_name(&value.archetype).len(),
            level_name(value.level).len(),
            value.title.len(),
            value.message.as_deref().map_or(0, str::len),
            value.owner.as_ref().map_or(0, |id| id.as_str().len()),
            value
                .template_identifier
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
            condition.saturating_sub(std::mem::size_of::<conditions::Condition>()),
        ];
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for field in &value.fields {
            bytes = bytes.saturating_add(Field::encoding_memory(field)?);
        }
        for action in &value.actions {
            bytes = bytes.saturating_add(Action::encoding_memory(action));
        }
        for rule in &value.rules {
            bytes = bytes.saturating_add(Rule::encoding_memory(rule)?);
        }
        for example in &value.examples {
            bytes = bytes.saturating_add(Map::encoding_memory(example)?);
        }
        Ok(bytes)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        archetype(&self.archetype)?;
        level(&self.level)?;
        let mut bytes = std::mem::size_of::<PromptDefinition>()
            .saturating_add(self.prompt_type.capacity())
            .saturating_add(self.title.capacity())
            .saturating_add(self.message.as_ref().map_or(0, String::capacity))
            .saturating_add(self.owner.as_ref().map_or(0, String::capacity))
            .saturating_add(
                self.template_identifier
                    .as_ref()
                    .map_or(0, String::capacity),
            );
        for field in &self.fields {
            bytes = bytes.saturating_add(field.projection_memory()?);
        }
        for action in &self.actions {
            bytes = bytes.saturating_add(action.projection_memory()?);
        }
        for rule in &self.rules {
            bytes = bytes.saturating_add(rule.projection_memory()?);
        }
        if let Some(condition) = &self.condition {
            bytes = bytes.saturating_add(condition.projection_memory()?);
        }
        for example in &self.examples {
            bytes = bytes.saturating_add(example.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&PromptDefinition> for Prompt {
    type Error = Error;

    fn try_from(value: &PromptDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("prompt encoding memory overflows"));
        }
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(value.fields.len())
            .map_err(|_| Error::Invalid("prompt fields allocation failed"))?;
        for field in &value.fields {
            fields.push(Field::try_from(field)?);
        }
        let mut actions = Vec::new();
        actions
            .try_reserve_exact(value.actions.len())
            .map_err(|_| Error::Invalid("prompt actions allocation failed"))?;
        for action in &value.actions {
            actions.push(Action::try_from(action)?);
        }
        let mut rules = Vec::new();
        rules
            .try_reserve_exact(value.rules.len())
            .map_err(|_| Error::Invalid("prompt rules allocation failed"))?;
        for rule in &value.rules {
            rules.push(Rule::try_from(rule)?);
        }
        let mut examples = Vec::new();
        examples
            .try_reserve_exact(value.examples.len())
            .map_err(|_| Error::Invalid("prompt examples allocation failed"))?;
        for example in &value.examples {
            examples.push(Map::try_from(example)?);
        }
        Ok(Self {
            prompt_type: value.prompt_type.as_str().to_owned(),
            schema_version: value.schema_version,
            archetype: archetype_name(&value.archetype).to_owned(),
            title: value.title.as_str().to_owned(),
            message: value.message.as_deref().map(str::to_owned),
            fields,
            actions,
            rules,
            condition: value
                .condition
                .as_ref()
                .map(conditions::Condition::try_from)
                .transpose()?,
            owner: value.owner.as_ref().map(|id| id.as_str().to_owned()),
            template_identifier: value
                .template_identifier
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            examples,
            level: level_name(value.level).to_owned(),
        })
    }
}

impl TryFrom<Prompt> for PromptDefinition {
    type Error = Error;

    fn try_from(row: Prompt) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("prompt projection memory overflows"));
        }
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(row.fields.len())
            .map_err(|_| Error::Invalid("prompt fields allocation failed"))?;
        for field in row.fields {
            fields.push(field.try_into()?);
        }
        let mut actions = Vec::new();
        actions
            .try_reserve_exact(row.actions.len())
            .map_err(|_| Error::Invalid("prompt actions allocation failed"))?;
        for action in row.actions {
            actions.push(action.try_into()?);
        }
        let mut rules = Vec::new();
        rules
            .try_reserve_exact(row.rules.len())
            .map_err(|_| Error::Invalid("prompt rules allocation failed"))?;
        for rule in row.rules {
            rules.push(rule.try_into()?);
        }
        let mut examples = Vec::new();
        examples
            .try_reserve_exact(row.examples.len())
            .map_err(|_| Error::Invalid("prompt examples allocation failed"))?;
        for example in row.examples {
            examples.push(example.try_into()?);
        }
        Ok(Self {
            prompt_type: dokime_domain::model::identity::Identifier::new(row.prompt_type)?,
            schema_version: row.schema_version,
            archetype: archetype(&row.archetype)?,
            title: row.title,
            message: row.message,
            fields,
            actions,
            rules,
            condition: row.condition.map(TryInto::try_into).transpose()?,
            owner: row.owner.map(dokime_domain::model::identity::SuiteId::from_uuid).transpose()?,
            template_identifier: row
                .template_identifier
                .map(dokime_domain::model::identity::Identifier::new)
                .transpose()?,
            examples,
            level: level(&row.level)?,
        })
    }
}

impl Field {
    fn encoding_memory(value: &PromptField) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let default = value
            .default
            .as_ref()
            .map(Value::encoding_memory)
            .transpose()?
            .unwrap_or(0);
        let validation = Validation::encoding_memory(&value.validation)?;
        if [metadata, default, validation].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            super::value_type_name(&value.value_type).len(),
            field_kind_name(&value.kind).len(),
            default.saturating_sub(std::mem::size_of::<Value>()),
            validation.saturating_sub(std::mem::size_of::<Validation>()),
            value
                .choices
                .len()
                .saturating_mul(std::mem::size_of::<String>()),
        ];
        let bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add);
        Ok(value
            .choices
            .iter()
            .fold(bytes, |bytes, choice| bytes.saturating_add(choice.len())))
    }

    fn projection_memory(&self) -> Result<usize> {
        super::value_type(&self.value_type)?;
        field_kind(&self.kind)?;
        let metadata = self
            .metadata
            .as_ref()
            .ok_or(Error::Invalid("prompt field metadata is missing"))?
            .projection_memory()?;
        let default = self
            .default
            .as_ref()
            .map(Value::projection_memory)
            .transpose()?
            .unwrap_or(0);
        use harmos::MemorySize;
        let choices = self.choices.memory_size();
        if choices == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<PromptField>()
            .saturating_add(metadata)
            .saturating_add(default)
            .saturating_add(choices)
            .saturating_add(self.validation.projection_memory()?))
    }
}

impl TryFrom<&PromptField> for Field {
    type Error = Error;

    fn try_from(value: &PromptField) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("prompt field encoding memory overflows"));
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            value_type: super::value_type_name(&value.value_type).to_owned(),
            kind: field_kind_name(&value.kind).to_owned(),
            multiple: value.multiple,
            required: value.required,
            default: value.default.as_ref().map(Value::try_from).transpose()?,
            choices: super::copied_strings(&value.choices)?,
            validation: Validation::try_from(&value.validation)?,
        })
    }
}

impl TryFrom<Field> for PromptField {
    type Error = Error;

    fn try_from(row: Field) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("prompt field projection memory overflows"));
        }
        Ok(Self {
            metadata: row
                .metadata
                .ok_or(Error::Invalid("prompt field metadata is missing"))?
                .try_into()?,
            value_type: super::value_type(&row.value_type)?,
            kind: field_kind(&row.kind)?,
            multiple: row.multiple,
            required: row.required,
            default: row.default.map(TryInto::try_into).transpose()?,
            choices: row.choices,
            validation: row.validation.try_into()?,
        })
    }
}

impl Action {
    fn encoding_memory(value: &PromptAction) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(value.id.len())
            .saturating_add(value.label.len())
            .saturating_add(emphasis_name(&value.emphasis).len())
    }

    fn projection_memory(&self) -> Result<usize> {
        emphasis(&self.emphasis)?;
        Ok(std::mem::size_of::<PromptAction>()
            .saturating_add(self.id.capacity())
            .saturating_add(self.label.capacity()))
    }
}

impl TryFrom<&PromptAction> for Action {
    type Error = Error;

    fn try_from(value: &PromptAction) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("prompt action encoding memory overflows"));
        }
        Ok(Self {
            id: value.id.as_str().to_owned(),
            label: value.label.as_str().to_owned(),
            emphasis: emphasis_name(&value.emphasis).to_owned(),
            requires_condition: value.requires_condition,
        })
    }
}

impl TryFrom<Action> for PromptAction {
    type Error = Error;

    fn try_from(row: Action) -> Result<Self> {
        Ok(Self {
            id: row.id,
            label: row.label,
            emphasis: emphasis(&row.emphasis)?,
            requires_condition: row.requires_condition,
        })
    }
}

impl Rule {
    fn encoding_memory(value: &PromptRule) -> Result<usize> {
        let predicate = expressions::Boolean::encoding_memory(&value.predicate)?;
        if predicate == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.message.len())
            .saturating_add(predicate.saturating_sub(std::mem::size_of::<expressions::Boolean>())))
    }

    fn projection_memory(&self) -> Result<usize> {
        Ok(std::mem::size_of::<PromptRule>()
            .saturating_add(self.message.capacity())
            .saturating_add(self.predicate.projection_memory()?))
    }
}

impl TryFrom<&PromptRule> for Rule {
    type Error = Error;

    fn try_from(value: &PromptRule) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("prompt rule encoding memory overflows"));
        }
        Ok(Self {
            predicate: expressions::Boolean::try_from(&value.predicate)?,
            message: value.message.as_str().to_owned(),
        })
    }
}

impl TryFrom<Rule> for PromptRule {
    type Error = Error;

    fn try_from(row: Rule) -> Result<Self> {
        Ok(Self {
            predicate: row.predicate.try_into()?,
            message: row.message,
        })
    }
}

impl Validation {
    fn encoding_memory(value: &FieldValidation) -> Result<usize> {
        for count in [
            value.min_length,
            value.max_length,
            value.min_items,
            value.max_items,
        ]
        .into_iter()
        .flatten()
        {
            encoded_length(count)?;
        }
        let minimum = value
            .minimum
            .as_ref()
            .map(Value::encoding_memory)
            .transpose()?
            .unwrap_or(0);
        let maximum = value
            .maximum
            .as_ref()
            .map(Value::encoding_memory)
            .transpose()?
            .unwrap_or(0);
        if [minimum, maximum].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.pattern.as_deref().map_or(0, str::len))
            .saturating_add(minimum.saturating_sub(std::mem::size_of::<Value>()))
            .saturating_add(maximum.saturating_sub(std::mem::size_of::<Value>())))
    }

    fn projection_memory(&self) -> Result<usize> {
        for count in [
            self.min_length,
            self.max_length,
            self.min_items,
            self.max_items,
        ]
        .into_iter()
        .flatten()
        {
            length(count)?;
        }
        let minimum = self
            .minimum
            .as_ref()
            .map(Value::projection_memory)
            .transpose()?
            .unwrap_or(0);
        let maximum = self
            .maximum
            .as_ref()
            .map(Value::projection_memory)
            .transpose()?
            .unwrap_or(0);
        Ok(std::mem::size_of::<FieldValidation>()
            .saturating_add(self.pattern.as_ref().map_or(0, String::capacity))
            .saturating_add(minimum)
            .saturating_add(maximum))
    }
}

impl TryFrom<&FieldValidation> for Validation {
    type Error = Error;

    fn try_from(value: &FieldValidation) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid(
                "prompt validation encoding memory overflows",
            ));
        }
        Ok(Self {
            min_length: value.min_length.map(encoded_length).transpose()?,
            max_length: value.max_length.map(encoded_length).transpose()?,
            pattern: value.pattern.as_deref().map(str::to_owned),
            minimum: value.minimum.as_ref().map(Value::try_from).transpose()?,
            maximum: value.maximum.as_ref().map(Value::try_from).transpose()?,
            min_items: value.min_items.map(encoded_length).transpose()?,
            max_items: value.max_items.map(encoded_length).transpose()?,
        })
    }
}

impl TryFrom<Validation> for FieldValidation {
    type Error = Error;

    fn try_from(row: Validation) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid(
                "prompt validation projection memory overflows",
            ));
        }
        Ok(Self {
            min_length: row.min_length.map(length).transpose()?,
            max_length: row.max_length.map(length).transpose()?,
            pattern: row.pattern,
            minimum: row.minimum.map(TryInto::try_into).transpose()?,
            maximum: row.maximum.map(TryInto::try_into).transpose()?,
            min_items: row.min_items.map(length).transpose()?,
            max_items: row.max_items.map(length).transpose()?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn encoded_length(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| Error::Invalid("prompt validation length overflows"))
}

fn archetype_name(value: &PromptArchetype) -> &'static str {
    match value {
        PromptArchetype::ConfirmationPrompt => "ConfirmationPrompt",
        PromptArchetype::FormPrompt => "FormPrompt",
        PromptArchetype::ChoicePrompt => "ChoicePrompt",
        PromptArchetype::StatusPrompt => "StatusPrompt",
    }
}

fn field_kind_name(value: &PromptFieldKind) -> &'static str {
    match value {
        PromptFieldKind::Text => "text",
        PromptFieldKind::Float => "float",
        PromptFieldKind::Int => "int",
        PromptFieldKind::Choice => "choice",
        PromptFieldKind::Boolean => "boolean",
    }
}

fn emphasis_name(value: &PromptActionEmphasis) -> &'static str {
    match value {
        PromptActionEmphasis::Primary => "primary",
        PromptActionEmphasis::Secondary => "secondary",
        PromptActionEmphasis::Danger => "danger",
    }
}

fn level_name(value: RecordSeverity) -> &'static str {
    match value {
        RecordSeverity::Debug => "debug",
        RecordSeverity::Info => "info",
        RecordSeverity::Warning => "warning",
        RecordSeverity::Error => "error",
    }
}

fn length(value: u64) -> Result<usize> {
    usize::try_from(value).map_err(|_| Error::Invalid("prompt validation length overflows"))
}

fn archetype(value: &str) -> Result<PromptArchetype> {
    match value {
        "ConfirmationPrompt" => Ok(PromptArchetype::ConfirmationPrompt),
        "FormPrompt" => Ok(PromptArchetype::FormPrompt),
        "ChoicePrompt" => Ok(PromptArchetype::ChoicePrompt),
        "StatusPrompt" => Ok(PromptArchetype::StatusPrompt),
        _ => Err(Error::Invalid("invalid prompt archetype")),
    }
}

fn level(value: &str) -> Result<RecordSeverity> {
    match value {
        "debug" => Ok(RecordSeverity::Debug),
        "" | "info" => Ok(RecordSeverity::Info),
        "warning" => Ok(RecordSeverity::Warning),
        "error" => Ok(RecordSeverity::Error),
        _ => Err(Error::Invalid("invalid prompt level")),
    }
}

fn field_kind(value: &str) -> Result<PromptFieldKind> {
    match value {
        "text" => Ok(PromptFieldKind::Text),
        "float" => Ok(PromptFieldKind::Float),
        "int" => Ok(PromptFieldKind::Int),
        "choice" => Ok(PromptFieldKind::Choice),
        "boolean" => Ok(PromptFieldKind::Boolean),
        _ => Err(Error::Invalid("invalid prompt field kind")),
    }
}

fn emphasis(value: &str) -> Result<PromptActionEmphasis> {
    match value {
        "primary" => Ok(PromptActionEmphasis::Primary),
        "secondary" => Ok(PromptActionEmphasis::Secondary),
        "danger" => Ok(PromptActionEmphasis::Danger),
        _ => Err(Error::Invalid("invalid prompt action emphasis")),
    }
}

/* === Tests === */

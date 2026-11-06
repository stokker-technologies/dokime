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
    FieldValidation, PromptAction, PromptActionEmphasis, PromptArchetype, PromptDefinition, PromptField, PromptFieldKind,
    Rule as PromptRule,
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

impl TryFrom<&PromptDefinition> for Prompt {
    type Error = Error;

    fn try_from(value: &PromptDefinition) -> Result<Self> {
        let mut fields = Vec::new();
        fields.try_reserve_exact(value.fields.len()).map_err(|_| Error::Invalid("prompt fields allocation failed"))?;
        for field in &value.fields {
            fields.push(Field::try_from(field)?);
        }
        let mut actions = Vec::new();
        actions.try_reserve_exact(value.actions.len()).map_err(|_| Error::Invalid("prompt actions allocation failed"))?;
        for action in &value.actions {
            actions.push(Action::try_from(action)?);
        }
        let mut rules = Vec::new();
        rules.try_reserve_exact(value.rules.len()).map_err(|_| Error::Invalid("prompt rules allocation failed"))?;
        for rule in &value.rules {
            rules.push(Rule::try_from(rule)?);
        }
        let mut examples = Vec::new();
        examples.try_reserve_exact(value.examples.len()).map_err(|_| Error::Invalid("prompt examples allocation failed"))?;
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
            condition: value.condition.as_ref().map(conditions::Condition::try_from).transpose()?,
            owner: value.owner.as_ref().map(|id| id.as_str().to_owned()),
            template_identifier: value.template_identifier.as_ref().map(|id| id.as_str().to_owned()),
            examples,
            level: level_name(value.level).to_owned(),
        })
    }
}

impl TryFrom<Prompt> for PromptDefinition {
    type Error = Error;

    fn try_from(row: Prompt) -> Result<Self> {
        let mut fields = Vec::new();
        fields.try_reserve_exact(row.fields.len()).map_err(|_| Error::Invalid("prompt fields allocation failed"))?;
        for field in row.fields {
            fields.push(field.try_into()?);
        }
        let mut actions = Vec::new();
        actions.try_reserve_exact(row.actions.len()).map_err(|_| Error::Invalid("prompt actions allocation failed"))?;
        for action in row.actions {
            actions.push(action.try_into()?);
        }
        let mut rules = Vec::new();
        rules.try_reserve_exact(row.rules.len()).map_err(|_| Error::Invalid("prompt rules allocation failed"))?;
        for rule in row.rules {
            rules.push(rule.try_into()?);
        }
        let mut examples = Vec::new();
        examples.try_reserve_exact(row.examples.len()).map_err(|_| Error::Invalid("prompt examples allocation failed"))?;
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
            template_identifier: row.template_identifier.map(dokime_domain::model::identity::Identifier::new).transpose()?,
            examples,
            level: level(&row.level)?,
        })
    }
}

impl TryFrom<&PromptField> for Field {
    type Error = Error;

    fn try_from(value: &PromptField) -> Result<Self> {
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
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("prompt field metadata is missing"))?.try_into()?,
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

impl TryFrom<&PromptAction> for Action {
    type Error = Error;

    fn try_from(value: &PromptAction) -> Result<Self> {
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

impl TryFrom<&PromptRule> for Rule {
    type Error = Error;

    fn try_from(value: &PromptRule) -> Result<Self> {
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

impl TryFrom<&FieldValidation> for Validation {
    type Error = Error;

    fn try_from(value: &FieldValidation) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::Prompt as PromptRow;
    use crate::prompts::{ConfirmationPrompt, Prompt, PromptDefinition};
    use crate::records::RecordSeverity;

    #[test]
    fn prompt_level_rows() {
        let expected = ConfirmationPrompt::new("fixture.level", "Raised").level(RecordSeverity::Warning);
        let expected = Prompt::definition(&expected);
        let row = PromptRow::try_from(&expected).unwrap();
        let restored = PromptDefinition::try_from(row.clone()).unwrap();

        let mut baseline = serde_json::to_value(&row).unwrap();
        baseline.as_object_mut().unwrap().remove("level");
        let baseline: PromptRow = serde_json::from_value(baseline).unwrap();
        let defaulted = PromptDefinition::try_from(baseline).unwrap();

        assert_eq!(restored, expected);
        assert_eq!(restored.level, RecordSeverity::Warning);
        assert_eq!(defaulted.level, RecordSeverity::Info);
    }
}

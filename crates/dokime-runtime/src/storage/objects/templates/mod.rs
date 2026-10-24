/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod analysis;
mod conditions;
pub(super) mod documents;
mod expressions;
mod layout;
pub(super) mod prompts;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use super::metadata;
use crate::storage::{Error, Result, values};
use dokime_domain::model::values::UnitFamily;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Definition {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    parameter_groups: Vec<Group>,
    #[serde(default)]
    test_case_templates: Vec<Template>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Template {
    #[serde(default)]
    test_case_template_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    parameter_groups: Vec<Group>,
    #[serde(default)]
    parameters: Vec<Parameter>,
    #[serde(default)]
    files: Vec<File>,
    #[serde(default)]
    document: Option<documents::Document>,
    #[serde(default)]
    phases: Vec<Phase>,
    #[serde(default)]
    criteria: Vec<Criterion>,
    #[serde(default)]
    prompt_definitions: Vec<prompts::Prompt>,
    #[serde(default)]
    subscriptions: Vec<Subscription>,
    #[serde(default)]
    attributes: Map,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Group {
    #[serde(default)]
    group_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Parameter {
    #[serde(default)]
    expansion: Expansion,
    #[serde(default)]
    parameter_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    group_id: Option<String>,
    #[serde(default)]
    value_type: String,
    #[serde(default)]
    unit: Option<Unit>,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    expression: expressions::Parameter,
    #[serde(default)]
    runtime_writable: bool,
    #[serde(default)]
    selection: Option<Selection>,
    #[serde(default)]
    list: Option<List>,
    #[serde(default)]
    extensions: Map,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Expansion {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    group: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Selection {
    #[serde(default)]
    choices: Vec<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct List {
    #[serde(default)]
    element_type: String,
    #[serde(default)]
    min_items: Option<u64>,
    #[serde(default)]
    max_items: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    family: String,
    #[serde(default)]
    symbol: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct File {
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    media_types: Vec<String>,
    #[serde(default)]
    attributes: Map,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Phase {
    #[serde(default)]
    repeatable: bool,
    #[serde(default)]
    step_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Criterion {
    #[serde(default)]
    criterion_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    expected: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Subscription {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    identifier: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Value {
    cells: Vec<values::Cell>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Map {
    cells: Vec<values::Cell>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Template {
    pub(super) fn encoding_memory(value: &crate::definition::TestCaseTemplate) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let attributes = Map::encoding_memory(&value.attributes)?;
        let document = value.document.as_ref().map(documents::Document::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, attributes, document].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            value.test_case_template_id.as_str().len(),
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            attributes.saturating_sub(std::mem::size_of::<Map>()),
            document.saturating_sub(std::mem::size_of::<documents::Document>()),
            value.tags.len().saturating_mul(std::mem::size_of::<String>()),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for tag in &value.tags {
            bytes = bytes.saturating_add(tag.len());
        }
        for group in &value.parameter_groups {
            bytes = bytes.saturating_add(Group::encoding_memory(group)?);
        }
        for parameter in &value.parameters {
            bytes = bytes.saturating_add(Parameter::encoding_memory(parameter)?);
        }
        for file in &value.files {
            bytes = bytes.saturating_add(File::encoding_memory(file)?);
        }
        for phase in &value.phases {
            bytes = bytes.saturating_add(Phase::encoding_memory(phase)?);
        }
        for criterion in &value.criteria {
            bytes = bytes.saturating_add(Criterion::encoding_memory(criterion)?);
        }
        for prompt in &value.prompt_definitions {
            bytes = bytes.saturating_add(prompts::Prompt::encoding_memory(prompt)?);
        }
        for subscription in &value.subscriptions {
            bytes = bytes.saturating_add(Subscription::encoding_memory(subscription));
        }
        Ok(bytes)
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("template metadata is missing"))?.projection_memory()?;
        let mut bytes = std::mem::size_of::<crate::definition::TestCaseTemplate>()
            .saturating_add(self.test_case_template_id.capacity())
            .saturating_add(metadata)
            .saturating_add(self.tags.memory_size())
            .saturating_add(self.attributes.projection_memory()?);
        if let Some(document) = &self.document {
            bytes = bytes.saturating_add(document.projection_memory()?);
        }
        for group in &self.parameter_groups {
            bytes = bytes.saturating_add(group.projection_memory()?);
        }
        for parameter in &self.parameters {
            bytes = bytes.saturating_add(parameter.projection_memory()?);
        }
        for file in &self.files {
            bytes = bytes.saturating_add(file.projection_memory()?);
        }
        for phase in &self.phases {
            bytes = bytes.saturating_add(phase.projection_memory()?);
        }
        for criterion in &self.criteria {
            bytes = bytes.saturating_add(criterion.projection_memory()?);
        }
        for prompt in &self.prompt_definitions {
            bytes = bytes.saturating_add(prompt.projection_memory()?);
        }
        for subscription in &self.subscriptions {
            bytes = bytes.saturating_add(subscription.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::definition::TestCaseTemplate> for Template {
    type Error = Error;

    fn try_from(value: &crate::definition::TestCaseTemplate) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("template encoding memory overflows"));
        }
        let mut parameter_groups = Vec::new();
        parameter_groups
            .try_reserve_exact(value.parameter_groups.len())
            .map_err(|_| Error::Invalid("template groups allocation failed"))?;
        for group in &value.parameter_groups {
            parameter_groups.push(Group::try_from(group)?);
        }
        let mut parameters = Vec::new();
        parameters.try_reserve_exact(value.parameters.len()).map_err(|_| Error::Invalid("template parameters allocation failed"))?;
        for parameter in &value.parameters {
            parameters.push(Parameter::try_from(parameter)?);
        }
        let mut files = Vec::new();
        files.try_reserve_exact(value.files.len()).map_err(|_| Error::Invalid("template files allocation failed"))?;
        for file in &value.files {
            files.push(File::try_from(file)?);
        }
        let mut phases = Vec::new();
        phases.try_reserve_exact(value.phases.len()).map_err(|_| Error::Invalid("template phases allocation failed"))?;
        for phase in &value.phases {
            phases.push(Phase::try_from(phase)?);
        }
        let mut criteria = Vec::new();
        criteria.try_reserve_exact(value.criteria.len()).map_err(|_| Error::Invalid("template criteria allocation failed"))?;
        for criterion in &value.criteria {
            criteria.push(Criterion::try_from(criterion)?);
        }
        let mut prompt_definitions = Vec::new();
        prompt_definitions
            .try_reserve_exact(value.prompt_definitions.len())
            .map_err(|_| Error::Invalid("template prompts allocation failed"))?;
        for prompt in &value.prompt_definitions {
            prompt_definitions.push(prompts::Prompt::try_from(prompt)?);
        }
        let mut subscriptions = Vec::new();
        subscriptions
            .try_reserve_exact(value.subscriptions.len())
            .map_err(|_| Error::Invalid("template subscriptions allocation failed"))?;
        for subscription in &value.subscriptions {
            subscriptions.push(Subscription::try_from(subscription)?);
        }
        Ok(Self {
            test_case_template_id: value.test_case_template_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            tags: copied_strings(&value.tags)?,
            parameter_groups,
            parameters,
            files,
            document: value.document.as_ref().map(documents::Document::try_from).transpose()?,
            phases,
            criteria,
            prompt_definitions,
            subscriptions,
            attributes: Map::try_from(&value.attributes)?,
        })
    }
}

impl TryFrom<Template> for crate::definition::TestCaseTemplate {
    type Error = Error;

    fn try_from(row: Template) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("template projection memory overflows"));
        }
        let mut parameter_groups = Vec::new();
        parameter_groups
            .try_reserve_exact(row.parameter_groups.len())
            .map_err(|_| Error::Invalid("template parameter groups allocation failed"))?;
        for value in row.parameter_groups {
            parameter_groups.push(value.try_into()?);
        }
        let mut parameters = Vec::new();
        parameters.try_reserve_exact(row.parameters.len()).map_err(|_| Error::Invalid("template parameters allocation failed"))?;
        for value in row.parameters {
            parameters.push(value.try_into()?);
        }
        let mut files = Vec::new();
        files.try_reserve_exact(row.files.len()).map_err(|_| Error::Invalid("template files allocation failed"))?;
        for value in row.files {
            files.push(value.try_into()?);
        }
        let mut phases = Vec::new();
        phases.try_reserve_exact(row.phases.len()).map_err(|_| Error::Invalid("template phases allocation failed"))?;
        for value in row.phases {
            phases.push(value.try_into()?);
        }
        let mut criteria = Vec::new();
        criteria.try_reserve_exact(row.criteria.len()).map_err(|_| Error::Invalid("template criteria allocation failed"))?;
        for value in row.criteria {
            criteria.push(value.try_into()?);
        }
        let mut prompt_definitions = Vec::new();
        prompt_definitions
            .try_reserve_exact(row.prompt_definitions.len())
            .map_err(|_| Error::Invalid("template prompts allocation failed"))?;
        for value in row.prompt_definitions {
            prompt_definitions.push(value.try_into()?);
        }
        let mut subscriptions = Vec::new();
        subscriptions.try_reserve_exact(row.subscriptions.len()).map_err(|_| Error::Invalid("template subscriptions allocation failed"))?;
        for value in row.subscriptions {
            subscriptions.push(value.try_into()?);
        }
        Ok(Self {
            test_case_template_id: dokime_domain::model::identity::TemplateId::from_uuid(row.test_case_template_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("template metadata is missing"))?.try_into()?,
            tags: row.tags,
            parameter_groups,
            parameters,
            files,
            document: row.document.map(TryInto::try_into).transpose()?,
            phases,
            criteria,
            prompt_definitions,
            subscriptions,
            attributes: row.attributes.try_into()?,
        })
    }
}

impl File {
    fn encoding_memory(value: &crate::definition::FileDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let attributes = Map::encoding_memory(&value.attributes)?;
        if [metadata, attributes].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let bytes = std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(value.default.as_deref().map_or(0, str::len))
            .saturating_add(value.media_types.len().saturating_mul(std::mem::size_of::<String>()))
            .saturating_add(attributes.saturating_sub(std::mem::size_of::<Map>()));
        Ok(value.media_types.iter().fold(bytes, |bytes, media| bytes.saturating_add(media.len())))
    }

    fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("file metadata is missing"))?.projection_memory()?;
        let media_types = self.media_types.memory_size();
        if media_types == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<crate::definition::FileDefinition>()
            .saturating_add(metadata)
            .saturating_add(self.default.as_ref().map_or(0, String::capacity))
            .saturating_add(media_types)
            .saturating_add(self.attributes.projection_memory()?))
    }
}

impl TryFrom<&crate::definition::FileDefinition> for File {
    type Error = Error;

    fn try_from(value: &crate::definition::FileDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("file definition encoding memory overflows"));
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            default: value.default.as_deref().map(str::to_owned),
            required: value.required,
            media_types: copied_strings(&value.media_types)?,
            attributes: Map::try_from(&value.attributes)?,
        })
    }
}

impl TryFrom<File> for crate::definition::FileDefinition {
    type Error = Error;

    fn try_from(row: File) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("file definition projection memory overflows"));
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("file metadata is missing"))?.try_into()?,
            default: row.default,
            required: row.required,
            media_types: row.media_types,
            attributes: row.attributes.try_into()?,
        })
    }
}

impl Phase {
    fn encoding_memory(value: &crate::definition::ExecutionPhaseDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        if metadata == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.step_id.as_str().len())
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>())))
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("phase metadata is missing"))?.projection_memory()?;
        Ok(std::mem::size_of::<crate::definition::ExecutionPhaseDefinition>()
            .saturating_add(self.step_id.capacity())
            .saturating_add(metadata))
    }
}

impl TryFrom<&crate::definition::ExecutionPhaseDefinition> for Phase {
    type Error = Error;

    fn try_from(value: &crate::definition::ExecutionPhaseDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("phase encoding memory overflows"));
        }
        Ok(Self {
            repeatable: value.repeatable,
            step_id: value.step_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
        })
    }
}

impl TryFrom<Phase> for crate::definition::ExecutionPhaseDefinition {
    type Error = Error;

    fn try_from(row: Phase) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("phase projection memory overflows"));
        }
        Ok(Self {
            step_id: dokime_domain::model::identity::StepId::from_uuid(row.step_id)?,
            repeatable: row.repeatable,
            metadata: row.metadata.ok_or(Error::Invalid("phase metadata is missing"))?.try_into()?,
        })
    }
}

impl Criterion {
    fn encoding_memory(value: &crate::definition::CriterionDefinition) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let expected = value.expected.as_ref().map(Value::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, expected].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.criterion_id.as_str().len())
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(expected.saturating_sub(std::mem::size_of::<Value>())))
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("criterion metadata is missing"))?.projection_memory()?;
        let expected = self.expected.as_ref().map(Value::projection_memory).transpose()?.unwrap_or(0);
        Ok(std::mem::size_of::<crate::definition::CriterionDefinition>()
            .saturating_add(self.criterion_id.capacity())
            .saturating_add(metadata)
            .saturating_add(expected))
    }
}

impl TryFrom<&crate::definition::CriterionDefinition> for Criterion {
    type Error = Error;

    fn try_from(value: &crate::definition::CriterionDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("criterion definition encoding memory overflows"));
        }
        Ok(Self {
            criterion_id: value.criterion_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            expected: value.expected.as_ref().map(Value::try_from).transpose()?,
        })
    }
}

impl TryFrom<Criterion> for crate::definition::CriterionDefinition {
    type Error = Error;

    fn try_from(row: Criterion) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("criterion definition projection memory overflows"));
        }
        Ok(Self {
            criterion_id: dokime_domain::model::identity::CriterionId::from_uuid(row.criterion_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("criterion metadata is missing"))?.try_into()?,
            expected: row.expected.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Subscription {
    fn encoding_memory(value: &crate::definition::Subscription) -> usize {
        use crate::definition::Subscription;
        let (kind, identifier) = match value {
            Subscription::Stream(id) => ("stream", id),
            Subscription::Record(id) => ("record", id),
        };
        std::mem::size_of::<Self>().saturating_add(kind.len()).saturating_add(identifier.as_str().len())
    }

    fn projection_memory(&self) -> Result<usize> {
        if !matches!(self.kind.as_str(), "stream" | "record") {
            return Err(Error::Invalid("invalid subscription kind"));
        }
        Ok(std::mem::size_of::<crate::definition::Subscription>().saturating_add(self.identifier.capacity()))
    }
}

impl TryFrom<&crate::definition::Subscription> for Subscription {
    type Error = Error;

    fn try_from(value: &crate::definition::Subscription) -> Result<Self> {
        use crate::definition::Subscription;
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("subscription encoding memory overflows"));
        }
        let (kind, identifier) = match value {
            Subscription::Stream(id) => ("stream", id),
            Subscription::Record(id) => ("record", id),
        };
        Ok(Self { kind: kind.to_owned(), identifier: identifier.as_str().to_owned() })
    }
}

impl TryFrom<Subscription> for crate::definition::Subscription {
    type Error = Error;

    fn try_from(row: Subscription) -> Result<Self> {
        row.projection_memory()?;
        let identifier = dokime_domain::model::identity::Identifier::new(row.identifier)?;
        match row.kind.as_str() {
            "stream" => Ok(Self::Stream(identifier)),
            "record" => Ok(Self::Record(identifier)),
            _ => Err(Error::Invalid("invalid subscription kind")),
        }
    }
}

impl Parameter {
    pub(super) fn encoding_memory(value: &crate::definition::ParameterDefinition) -> Result<usize> {
        if [value.min, value.max].into_iter().flatten().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("parameter bounds are nonfinite"));
        }
        let expansion = Expansion::encoding_memory(&value.expansion);
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let unit = value.unit.as_ref().map(Unit::encoding_memory).transpose()?.unwrap_or(0);
        let expression = expressions::Parameter::encoding_memory(&value.expression)?;
        let selection = value.selection.as_ref().map(Selection::encoding_memory).transpose()?.unwrap_or(0);
        let list = value.list.as_ref().map(List::encoding_memory).transpose()?.unwrap_or(0);
        let extensions = Map::encoding_memory(&value.extensions)?;
        if [expansion, metadata, unit, expression, selection, list, extensions].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            expansion.saturating_sub(std::mem::size_of::<Expansion>()),
            value.parameter_id.as_str().len(),
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            value.group_id.as_ref().map_or(0, |id| id.as_str().len()),
            value_type_name(&value.value_type).len(),
            unit.saturating_sub(std::mem::size_of::<Unit>()),
            expression.saturating_sub(std::mem::size_of::<expressions::Parameter>()),
            selection.saturating_sub(std::mem::size_of::<Selection>()),
            list.saturating_sub(std::mem::size_of::<List>()),
            extensions.saturating_sub(std::mem::size_of::<Map>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        if [self.min, self.max].into_iter().flatten().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("parameter bounds are nonfinite"));
        }
        value_type(&self.value_type)?;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("parameter definition metadata is missing"))?.projection_memory()?;
        let fields = [
            self.parameter_id.capacity(),
            self.group_id.as_ref().map_or(0, String::capacity),
            self.expansion.projection_memory()?,
            metadata,
            self.unit.as_ref().map(Unit::projection_memory).transpose()?.unwrap_or(0),
            self.expression.projection_memory()?,
            self.selection.as_ref().map(Selection::projection_memory).transpose()?.unwrap_or(0),
            self.extensions.projection_memory()?,
        ];
        if let Some(list) = &self.list {
            list.validate()?;
        }
        Ok(fields.into_iter().fold(std::mem::size_of::<crate::definition::ParameterDefinition>(), usize::saturating_add))
    }
}

impl TryFrom<&crate::definition::ParameterDefinition> for Parameter {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterDefinition) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter definition encoding memory overflows"));
        }
        Ok(Self {
            expansion: Expansion::try_from(&value.expansion)?,
            parameter_id: value.parameter_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            group_id: value.group_id.as_ref().map(|id| id.as_str().to_owned()),
            value_type: value_type_name(&value.value_type).to_owned(),
            unit: value.unit.as_ref().map(Unit::try_from).transpose()?,
            min: value.min,
            max: value.max,
            required: value.required,
            expression: expressions::Parameter::try_from(&value.expression)?,
            runtime_writable: value.runtime_writable,
            selection: value.selection.as_ref().map(Selection::try_from).transpose()?,
            list: value.list.as_ref().map(List::try_from).transpose()?,
            extensions: Map::try_from(&value.extensions)?,
        })
    }
}

impl TryFrom<Parameter> for crate::definition::ParameterDefinition {
    type Error = Error;

    fn try_from(row: Parameter) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("parameter definition projection memory overflows"));
        }
        Ok(Self {
            expansion: row.expansion.try_into()?,
            parameter_id: dokime_domain::model::identity::ParameterId::from_uuid(row.parameter_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("parameter definition metadata is missing"))?.try_into()?,
            group_id: row.group_id.map(dokime_domain::model::identity::Identifier::new).transpose()?,
            value_type: value_type(&row.value_type)?,
            unit: row.unit.map(TryInto::try_into).transpose()?,
            min: row.min,
            max: row.max,
            required: row.required,
            expression: row.expression.try_into()?,
            runtime_writable: row.runtime_writable,
            selection: row.selection.map(TryInto::try_into).transpose()?,
            list: row.list.map(TryInto::try_into).transpose()?,
            extensions: row.extensions.try_into()?,
        })
    }
}

impl Expansion {
    fn encoding_memory(value: &crate::definition::ParameterExpansion) -> usize {
        use crate::definition::ParameterExpansion;
        let (kind, group) = match value {
            ParameterExpansion::Single => ("single", 0),
            ParameterExpansion::Cartesian => ("cartesian", 0),
            ParameterExpansion::Zip { group } => ("zip", group.as_str().len()),
        };
        std::mem::size_of::<Self>().saturating_add(kind.len()).saturating_add(group)
    }

    fn projection_memory(&self) -> Result<usize> {
        match (self.kind.as_str(), self.group.as_ref()) {
            ("single" | "cartesian", None) => Ok(0),
            ("zip", Some(group)) => Ok(group.capacity()),
            _ => Err(Error::Invalid("parameter expansion fields differ")),
        }
    }
}

impl TryFrom<&crate::definition::ParameterExpansion> for Expansion {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterExpansion) -> Result<Self> {
        use crate::definition::ParameterExpansion;
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("parameter expansion encoding memory overflows"));
        }
        let (kind, group) = match value {
            ParameterExpansion::Single => ("single", None),
            ParameterExpansion::Cartesian => ("cartesian", None),
            ParameterExpansion::Zip { group } => ("zip", Some(group.as_str().to_owned())),
        };
        Ok(Self { kind: kind.to_owned(), group })
    }
}

impl TryFrom<Expansion> for crate::definition::ParameterExpansion {
    type Error = Error;

    fn try_from(row: Expansion) -> Result<Self> {
        row.projection_memory()?;
        match row.kind.as_str() {
            "single" => Ok(Self::Single),
            "cartesian" => Ok(Self::Cartesian),
            "zip" => Ok(Self::Zip { group: dokime_domain::model::identity::Identifier::new(row.group.unwrap())? }),
            _ => Err(Error::Invalid("invalid parameter expansion")),
        }
    }
}

impl Selection {
    fn encoding_memory(value: &crate::definition::ParameterSelection) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>().saturating_add(value.choices.len().saturating_mul(std::mem::size_of::<Value>()));
        for choice in &value.choices {
            let memory = Value::encoding_memory(choice)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<Value>()));
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let mut bytes = 0usize;
        for choice in &self.choices {
            bytes = bytes.saturating_add(choice.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::definition::ParameterSelection> for Selection {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterSelection) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter choices encoding memory overflows"));
        }
        let mut choices = Vec::new();
        choices.try_reserve_exact(value.choices.len()).map_err(|_| Error::Invalid("parameter choices allocation failed"))?;
        for choice in &value.choices {
            choices.push(Value::try_from(choice)?);
        }
        Ok(Self { choices })
    }
}

impl TryFrom<Selection> for crate::definition::ParameterSelection {
    type Error = Error;

    fn try_from(row: Selection) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("parameter choices projection memory overflows"));
        }
        let mut choices = Vec::new();
        choices.try_reserve_exact(row.choices.len()).map_err(|_| Error::Invalid("parameter choices allocation failed"))?;
        for choice in row.choices {
            choices.push(choice.try_into()?);
        }
        Ok(Self { choices })
    }
}

impl List {
    fn encoding_memory(value: &crate::definition::ParameterList) -> Result<usize> {
        for count in [value.min_items, value.max_items].into_iter().flatten() {
            u64::try_from(count).map_err(|_| Error::Invalid("parameter list count overflows"))?;
        }
        Ok(std::mem::size_of::<Self>().saturating_add(value_type_name(&value.element_type).len()))
    }

    fn validate(&self) -> Result<()> {
        value_type(&self.element_type)?;
        for count in [self.min_items, self.max_items].into_iter().flatten() {
            usize::try_from(count).map_err(|_| Error::Invalid("parameter list count overflows"))?;
        }
        Ok(())
    }
}

impl TryFrom<&crate::definition::ParameterList> for List {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterList) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter list encoding memory overflows"));
        }
        Ok(Self {
            element_type: value_type_name(&value.element_type).to_owned(),
            min_items: value.min_items.map(u64::try_from).transpose().map_err(|_| Error::Invalid("parameter list count overflows"))?,
            max_items: value.max_items.map(u64::try_from).transpose().map_err(|_| Error::Invalid("parameter list count overflows"))?,
        })
    }
}

impl TryFrom<List> for crate::definition::ParameterList {
    type Error = Error;

    fn try_from(row: List) -> Result<Self> {
        row.validate()?;
        Ok(Self {
            element_type: value_type(&row.element_type)?,
            min_items: row.min_items.map(usize::try_from).transpose().map_err(|_| Error::Invalid("parameter list count overflows"))?,
            max_items: row.max_items.map(usize::try_from).transpose().map_err(|_| Error::Invalid("parameter list count overflows"))?,
        })
    }
}

impl Group {
    fn encoding_memory(value: &crate::definition::ParameterGroup) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        if metadata == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.group_id.as_str().len())
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>())))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("parameter group metadata is missing"))?.projection_memory()?;
        Ok(std::mem::size_of::<crate::definition::ParameterGroup>().saturating_add(self.group_id.capacity()).saturating_add(metadata))
    }
}

impl TryFrom<&crate::definition::ParameterGroup> for Group {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterGroup) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter group encoding memory overflows"));
        }
        Ok(Self { group_id: value.group_id.as_str().to_owned(), metadata: Some(metadata::Row::try_from(&value.metadata)?) })
    }
}

impl TryFrom<Group> for crate::definition::ParameterGroup {
    type Error = Error;

    fn try_from(row: Group) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("parameter group projection memory overflows"));
        }
        Ok(Self {
            group_id: dokime_domain::model::identity::Identifier::new(row.group_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("parameter group metadata is missing"))?.try_into()?,
        })
    }
}

impl TryFrom<&dokime_domain::model::values::MeasurementUnit> for Unit {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::values::MeasurementUnit) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("unit encoding memory overflow"));
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            family: unit_family(&value.family).to_owned(),
            symbol: value.symbol.as_str().to_owned(),
        })
    }
}

impl Unit {
    pub fn encoding_memory(value: &dokime_domain::model::values::MeasurementUnit) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        if metadata == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(unit_family(&value.family).len())
            .saturating_add(value.symbol.len()))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("unit metadata is missing"))?.projection_memory()?;
        if metadata == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<dokime_domain::model::values::MeasurementUnit>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()))
            .saturating_add(self.symbol.capacity()))
    }
}

impl TryFrom<Unit> for dokime_domain::model::values::MeasurementUnit {
    type Error = Error;

    fn try_from(row: Unit) -> Result<Self> {
        use dokime_domain::model::values::UnitFamily;
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("unit projection memory overflows"));
        }
        let family = match row.family.as_str() {
            "count" => UnitFamily::Count,
            "distance" => UnitFamily::Distance,
            "duration" => UnitFamily::Duration,
            "frequency" => UnitFamily::Frequency,
            "power" => UnitFamily::Power,
            "temperature" => UnitFamily::Temperature,
            "voltage" => UnitFamily::Voltage,
            "ratio" => UnitFamily::Ratio,
            "custom" => UnitFamily::Custom,
            _ => return Err(Error::Invalid("invalid unit family")),
        };
        Ok(Self { metadata: row.metadata.ok_or(Error::Invalid("unit metadata is missing"))?.try_into()?, family, symbol: row.symbol })
    }
}

impl TryFrom<&dokime_domain::model::values::Value> for Value {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::values::Value) -> Result<Self> {
        let cells = values::encode_value(value)?;
        Ok(Self { cells })
    }
}

impl TryFrom<Value> for dokime_domain::model::values::Value {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        let bound = value.projection_memory()?;
        let allowance = NonZeroUsize::new(bound).ok_or(Error::Invalid("scalar projection is empty"))?;
        let mut values = values::decode_owned(value.cells, allowance)?;
        if values.len() != 1 {
            return Err(Error::Invalid("scalar value must have one member"));
        }
        values.remove("value").ok_or(Error::Invalid("scalar value is missing"))
    }
}
impl Value {
    pub fn encoding_memory(value: &dokime_domain::model::values::Value) -> Result<usize> {
        values::value_encoding_memory(value)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let bound = values::owned_memory(&self.cells)?;
        if self.cells[0].children != 1 || self.cells.get(1).and_then(|cell| cell.name.as_deref()) != Some("value") {
            return Err(Error::Invalid("scalar value must have one value member"));
        }
        Ok(bound)
    }
}

impl Map {
    pub fn encoding_memory(value: &std::collections::BTreeMap<String, dokime_domain::model::values::Value>) -> Result<usize> {
        values::encoding_memory(value)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        values::owned_memory(&self.cells)
    }
}

impl TryFrom<&std::collections::BTreeMap<String, dokime_domain::model::values::Value>> for Map {
    type Error = Error;

    fn try_from(value: &std::collections::BTreeMap<String, dokime_domain::model::values::Value>) -> Result<Self> {
        Ok(Self { cells: values::encode(value)? })
    }
}

impl TryFrom<Map> for std::collections::BTreeMap<String, dokime_domain::model::values::Value> {
    type Error = Error;

    fn try_from(map: Map) -> Result<Self> {
        let bound = map.projection_memory()?;
        let allowance = NonZeroUsize::new(bound).ok_or(Error::Invalid("map projection is empty"))?;
        values::decode_owned(map.cells, allowance)
    }
}

impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let value: dokime_domain::model::values::Value = self.clone().try_into().map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = <dokime_domain::model::values::Value as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(&value).map_err(serde::de::Error::custom)
    }
}
impl serde::Serialize for Map {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let values: std::collections::BTreeMap<String, dokime_domain::model::values::Value> =
            self.clone().try_into().map_err(serde::ser::Error::custom)?;
        values.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for Map {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value =
            <std::collections::BTreeMap<String, dokime_domain::model::values::Value> as serde::Deserialize>::deserialize(deserializer)?;
        let cells = values::encode(&value).map_err(serde::de::Error::custom)?;
        Ok(Self { cells })
    }
}

/* === Public Functions === */
pub(super) fn normalize_document(document: &mut crate::document::DocumentDefinition) {
    let cards = document.cards.iter_mut().map(|card| &mut card.body);
    let section_cards = document.sections.iter_mut().flat_map(|section| &mut section.cards).map(|card| &mut card.body);
    for body in cards.chain(section_cards) {
        if let crate::document::DocumentCardBody::Plot { plot } = body {
            // This Builder Flag Is Excluded From The Portable Contract
            plot.automatic_id = false;
        }
    }
}

pub(super) fn convert<T: serde::Serialize, U: serde::de::DeserializeOwned>(source: &T) -> Result<U> {
    let value = serde_json::to_value(source)?;
    Ok(serde_json::from_value(value)?)
}

/* === Private Functions === */
fn value_type_name(value: &dokime_domain::model::values::ValueType) -> &'static str {
    use dokime_domain::model::values::ValueType;
    match value {
        ValueType::Boolean => "boolean",
        ValueType::Integer => "integer",
        ValueType::Number => "number",
        ValueType::Text => "text",
        ValueType::Datetime => "datetime",
        ValueType::File => "file",
        ValueType::Object => "object",
        ValueType::Array => "array",
    }
}

fn copied_strings(values: &[String]) -> Result<Vec<String>> {
    let mut copied = Vec::new();
    copied.try_reserve_exact(values.len()).map_err(|_| Error::Invalid("template text allocation failed"))?;
    for value in values {
        copied.push(value.as_str().to_owned());
    }
    Ok(copied)
}

fn unit_family(family: &dokime_domain::model::values::UnitFamily) -> &'static str {
    match family {
        UnitFamily::Count => "count",
        UnitFamily::Distance => "distance",
        UnitFamily::Duration => "duration",
        UnitFamily::Frequency => "frequency",
        UnitFamily::Power => "power",
        UnitFamily::Temperature => "temperature",
        UnitFamily::Voltage => "voltage",
        UnitFamily::Ratio => "ratio",
        UnitFamily::Custom => "custom",
    }
}

fn value_type(value: &str) -> Result<dokime_domain::model::values::ValueType> {
    use dokime_domain::model::values::ValueType;
    match value {
        "boolean" => Ok(ValueType::Boolean),
        "integer" => Ok(ValueType::Integer),
        "number" => Ok(ValueType::Number),
        "text" => Ok(ValueType::Text),
        "datetime" => Ok(ValueType::Datetime),
        "file" => Ok(ValueType::File),
        "object" => Ok(ValueType::Object),
        "array" => Ok(ValueType::Array),
        _ => Err(Error::Invalid("invalid parameter value type")),
    }
}

/* === Tests === */

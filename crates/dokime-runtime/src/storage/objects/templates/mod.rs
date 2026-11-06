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
mod suite;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

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
#[derive(Clone, PartialEq)]
pub(super) struct Definition {
    metadata: Option<metadata::Row>,
    tags: Vec<String>,
    parameter_groups: Vec<Group>,
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

impl TryFrom<&crate::definition::TestCaseTemplate> for Template {
    type Error = Error;

    fn try_from(value: &crate::definition::TestCaseTemplate) -> Result<Self> {
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

impl TryFrom<&crate::definition::FileDefinition> for File {
    type Error = Error;

    fn try_from(value: &crate::definition::FileDefinition) -> Result<Self> {
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
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("file metadata is missing"))?.try_into()?,
            default: row.default,
            required: row.required,
            media_types: row.media_types,
            attributes: row.attributes.try_into()?,
        })
    }
}

impl TryFrom<&crate::definition::ExecutionPhaseDefinition> for Phase {
    type Error = Error;

    fn try_from(value: &crate::definition::ExecutionPhaseDefinition) -> Result<Self> {
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
        Ok(Self {
            step_id: dokime_domain::model::identity::StepId::from_uuid(row.step_id)?,
            repeatable: row.repeatable,
            metadata: row.metadata.ok_or(Error::Invalid("phase metadata is missing"))?.try_into()?,
        })
    }
}

impl TryFrom<&crate::definition::CriterionDefinition> for Criterion {
    type Error = Error;

    fn try_from(value: &crate::definition::CriterionDefinition) -> Result<Self> {
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
        Ok(Self {
            criterion_id: dokime_domain::model::identity::CriterionId::from_uuid(row.criterion_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("criterion metadata is missing"))?.try_into()?,
            expected: row.expected.map(TryInto::try_into).transpose()?,
        })
    }
}

impl TryFrom<&crate::definition::Subscription> for Subscription {
    type Error = Error;

    fn try_from(value: &crate::definition::Subscription) -> Result<Self> {
        use crate::definition::Subscription;
        let (kind, identifier) = match value {
            Subscription::Stream(id) => ("stream", id),
            Subscription::Record(id) => ("record", id),
        };
        Ok(Self {
            kind: kind.to_owned(),
            identifier: identifier.as_str().to_owned(),
        })
    }
}

impl TryFrom<Subscription> for crate::definition::Subscription {
    type Error = Error;

    fn try_from(row: Subscription) -> Result<Self> {
        let identifier = dokime_domain::model::identity::Identifier::new(row.identifier)?;
        match row.kind.as_str() {
            "stream" => Ok(Self::Stream(identifier)),
            "record" => Ok(Self::Record(identifier)),
            _ => Err(Error::Invalid("invalid subscription kind")),
        }
    }
}

impl TryFrom<&crate::definition::ParameterDefinition> for Parameter {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterDefinition) -> Result<Self> {
        finite_bounds(value.min, value.max)?;
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
        finite_bounds(row.min, row.max)?;
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

impl TryFrom<&crate::definition::ParameterExpansion> for Expansion {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterExpansion) -> Result<Self> {
        use crate::definition::ParameterExpansion;
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
        match (row.kind.as_str(), row.group) {
            ("single", None) => Ok(Self::Single),
            ("cartesian", None) => Ok(Self::Cartesian),
            ("zip", Some(group)) => Ok(Self::Zip {
                group: dokime_domain::model::identity::Identifier::new(group)?,
            }),
            ("single" | "cartesian" | "zip", _) => Err(Error::Invalid("parameter expansion fields differ")),
            _ => Err(Error::Invalid("invalid parameter expansion")),
        }
    }
}

impl TryFrom<&crate::definition::ParameterSelection> for Selection {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterSelection) -> Result<Self> {
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
        let mut choices = Vec::new();
        choices.try_reserve_exact(row.choices.len()).map_err(|_| Error::Invalid("parameter choices allocation failed"))?;
        for choice in row.choices {
            choices.push(choice.try_into()?);
        }
        Ok(Self { choices })
    }
}

impl List {
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

impl TryFrom<&crate::definition::ParameterGroup> for Group {
    type Error = Error;

    fn try_from(value: &crate::definition::ParameterGroup) -> Result<Self> {
        Ok(Self {
            group_id: value.group_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
        })
    }
}

impl TryFrom<Group> for crate::definition::ParameterGroup {
    type Error = Error;

    fn try_from(row: Group) -> Result<Self> {
        Ok(Self {
            group_id: dokime_domain::model::identity::Identifier::new(row.group_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("parameter group metadata is missing"))?.try_into()?,
        })
    }
}

impl TryFrom<&dokime_domain::model::values::MeasurementUnit> for Unit {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::values::MeasurementUnit) -> Result<Self> {
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            family: unit_family(&value.family).to_owned(),
            symbol: value.symbol.as_str().to_owned(),
        })
    }
}

impl TryFrom<Unit> for dokime_domain::model::values::MeasurementUnit {
    type Error = Error;

    fn try_from(row: Unit) -> Result<Self> {
        use dokime_domain::model::values::UnitFamily;
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
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("unit metadata is missing"))?.try_into()?,
            family,
            symbol: row.symbol,
        })
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
        let mut values = values::decode_owned(value.cells)?;
        if values.len() != 1 {
            return Err(Error::Invalid("scalar value must have one member"));
        }
        values.remove("value").ok_or(Error::Invalid("scalar value is missing"))
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
        values::decode_owned(map.cells)
    }
}

impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let value = values::decode_value(&self.cells).map_err(serde::ser::Error::custom)?;
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
        values::map::serialize(&self.cells, serializer)
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

fn finite_bounds(min: Option<f64>, max: Option<f64>) -> Result<()> {
    if [min, max].into_iter().flatten().any(|value| !value.is_finite()) {
        return Err(Error::Invalid("parameter bounds are nonfinite"));
    }
    Ok(())
}

fn exact_slots<T>(length: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(length).map_err(|_| Error::Invalid("suite allocation failed"))?;
    Ok(values)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn suite_definition() -> Definition {
        use dokime_domain::model::identity::{Identifier, Metadata};
        let source = crate::definition::TestSuite {
            metadata: Metadata::new(Identifier::new("fixture.suite").unwrap(), "Fixture", None),
            tags: vec!["second".into(), "first".into()],
            parameter_groups: Vec::new(),
            test_case_templates: Vec::new(),
        };
        Definition::try_from(&source).unwrap()
    }

    #[test]
    fn stored_suite_rejects_missing_metadata_and_noncanonical_time() {
        let mut missing = suite_definition();
        missing.metadata = None;
        assert!(crate::definition::TestSuite::try_from(missing).is_err());

        let mut invalid = suite_definition();
        invalid.metadata.as_mut().unwrap().created_at = Some("2026-10-01T03:00:00+03:00".into());
        assert!(crate::definition::TestSuite::try_from(invalid).is_err());
    }

    #[test]
    fn stored_suite_rejects_duplicate_metadata_tags() {
        let mut invalid = suite_definition();
        invalid.metadata.as_mut().unwrap().tags = vec!["same".into(), "same".into()];
        assert!(crate::definition::TestSuite::try_from(invalid).is_err());
    }

    #[test]
    fn cell_adapters_preserve_native_tagged_serde_format() {
        use dokime_domain::model::values::Value as DomainValue;
        let datetime = time::OffsetDateTime::parse("1999-12-31T23:59:59.123Z", &time::format_description::well_known::Rfc3339).unwrap();
        let entries = std::collections::BTreeMap::from([
            ("array".into(), DomainValue::Array(vec![DomainValue::Null, DomainValue::Bool(true), DomainValue::Number(-0.0)])),
            ("integer".into(), DomainValue::Integer(i64::MIN)),
            ("unsigned".into(), DomainValue::Unsigned(u64::MAX)),
            ("datetime".into(), DomainValue::Datetime(datetime)),
            ("text".into(), DomainValue::Text("payload".into())),
            ("empty".into(), DomainValue::Object(std::collections::BTreeMap::new())),
        ]);
        let map = Map::try_from(&entries).unwrap();
        let encoded = serde_json::to_string(&map).unwrap();
        assert_eq!(encoded, serde_json::to_string(&entries).unwrap());
        let restored: Map = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.cells, map.cells);

        for value in entries.values() {
            let row = Value::try_from(value).unwrap();
            let encoded = serde_json::to_string(&row).unwrap();
            assert_eq!(encoded, serde_json::to_string(value).unwrap());
            let restored: Value = serde_json::from_str(&encoded).unwrap();
            assert_eq!(restored.cells, row.cells);
        }
        let mut invalid = Value::try_from(&DomainValue::Null).unwrap();
        invalid.cells[1].name = Some("other".into());
        assert!(serde_json::to_string(&invalid).is_err());
    }
}

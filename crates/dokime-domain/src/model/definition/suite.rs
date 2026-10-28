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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::definition::ParameterDefinition;
use crate::model::definition::ParameterGroup;
use crate::model::document::DocumentDefinition;
use crate::model::identity::CaseId;
use crate::model::identity::CriterionId;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::StepId;
use crate::model::identity::TemplateId;
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestSuite {
    pub metadata: Metadata,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub parameter_groups: Vec<ParameterGroup>,

    #[serde(default)]
    pub test_case_templates: Vec<TestCaseTemplate>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCaseTemplate {
    pub test_case_template_id: TemplateId,
    pub metadata: Metadata,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub parameter_groups: Vec<ParameterGroup>,

    pub parameters: Vec<ParameterDefinition>,

    #[serde(default)]
    pub files: Vec<FileDefinition>,

    #[serde(default)]
    pub document: Option<DocumentDefinition>,

    #[serde(default)]
    pub phases: Vec<ExecutionPhaseDefinition>,

    #[serde(default)]
    pub criteria: Vec<CriterionDefinition>,

    #[serde(default)]
    pub prompt_definitions: Vec<crate::model::prompts::PromptDefinition>,

    #[serde(default)]
    pub subscriptions: Vec<Subscription>,

    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    pub suite_id: String,
    pub test_case_id: CaseId,
    pub test_case_template_id: TemplateId,
    pub metadata: Metadata,

    #[serde(default)]
    pub suite: Option<Metadata>,

    #[serde(default)]
    pub parameters: BTreeMap<String, Value>,

    #[serde(default)]
    pub generation_index: u64,

    #[serde(default)]
    pub label_number: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileDefinition {
    pub metadata: Metadata,

    #[serde(default)]
    pub default: Option<String>,

    #[serde(default)]
    pub required: bool,

    #[serde(default)]
    pub media_types: Vec<String>,

    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPhaseDefinition {
    pub step_id: StepId,

    #[serde(default)]
    pub repeatable: bool,

    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CriterionDefinition {
    pub criterion_id: CriterionId,
    pub metadata: Metadata,
    pub expected: Option<Value>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "identifier", rename_all = "snake_case")]
pub enum Subscription {
    Stream(Identifier),
    Record(Identifier),
}

// Why a template identifier is not its suite's public reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateIdentifierRefusal {
    OutsideSuite,
    InvalidSegment,
}

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for TestCaseTemplate {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.test_case_template_id), std::mem::size_of_val(&self.test_case_template_id)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.tags), std::mem::size_of_val(&self.tags)),
            (harmos::MemorySize::memory_size(&self.parameter_groups), std::mem::size_of_val(&self.parameter_groups)),
            (harmos::MemorySize::memory_size(&self.parameters), std::mem::size_of_val(&self.parameters)),
            (harmos::MemorySize::memory_size(&self.files), std::mem::size_of_val(&self.files)),
            (harmos::MemorySize::memory_size(&self.document), std::mem::size_of_val(&self.document)),
            (harmos::MemorySize::memory_size(&self.phases), std::mem::size_of_val(&self.phases)),
            (harmos::MemorySize::memory_size(&self.criteria), std::mem::size_of_val(&self.criteria)),
            (harmos::MemorySize::memory_size(&self.prompt_definitions), std::mem::size_of_val(&self.prompt_definitions)),
            (harmos::MemorySize::memory_size(&self.subscriptions), std::mem::size_of_val(&self.subscriptions)),
            (harmos::MemorySize::memory_size(&self.attributes), std::mem::size_of_val(&self.attributes)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for FileDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.default), std::mem::size_of_val(&self.default)),
            (harmos::MemorySize::memory_size(&self.media_types), std::mem::size_of_val(&self.media_types)),
            (harmos::MemorySize::memory_size(&self.attributes), std::mem::size_of_val(&self.attributes)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for ExecutionPhaseDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.step_id), std::mem::size_of_val(&self.step_id)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for CriterionDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.criterion_id), std::mem::size_of_val(&self.criterion_id)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.expected), std::mem::size_of_val(&self.expected)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for Subscription {
    fn memory_size(&self) -> usize {
        let identifier = match self {
            Self::Stream(identifier) | Self::Record(identifier) => identifier,
        };
        let owned = harmos::MemorySize::memory_size(identifier);
        if owned == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>().saturating_add(owned.saturating_sub(std::mem::size_of_val(identifier)))
    }
}

impl harmos::MemorySize for TestCase {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.suite_id.memory_size(), std::mem::size_of_val(&self.suite_id)),
            (self.test_case_id.memory_size(), std::mem::size_of_val(&self.test_case_id)),
            (self.test_case_template_id.memory_size(), std::mem::size_of_val(&self.test_case_template_id)),
            (self.metadata.memory_size(), std::mem::size_of_val(&self.metadata)),
            (self.suite.memory_size(), std::mem::size_of_val(&self.suite)),
            (self.parameters.memory_size(), std::mem::size_of_val(&self.parameters)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl Subscription {
    pub fn stream(&self) -> Option<&Identifier> {
        match self {
            Self::Stream(identifier) => Some(identifier),
            Self::Record(_) => None,
        }
    }

    pub fn record(&self) -> Option<&Identifier> {
        match self {
            Self::Record(identifier) => Some(identifier),
            Self::Stream(_) => None,
        }
    }
}

impl TemplateIdentifierRefusal {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::OutsideSuite => "a template id is its suite's id followed by `.` and one segment",
            Self::InvalidSegment => {
                "a template id's last segment is lowercase ASCII letters, digits, and underscores, starting with a letter"
            }
        }
    }
}

impl std::fmt::Display for TemplateIdentifierRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.reason())
    }
}

impl std::error::Error for TemplateIdentifierRefusal {}

/* === Public Functions === */

// The public template reference is `<suite id>.<segment>`: the suite owns the
// prefix and the segment is `[a-z][a-z0-9_]*`. Const so the SDK refuses a
// foreign or malformed id while the plugin compiles.
pub const fn template_identifier(suite: &str, template: &str) -> Result<(), TemplateIdentifierRefusal> {
    let (suite, template) = (suite.as_bytes(), template.as_bytes());
    if suite.is_empty() || template.len() <= suite.len() + 1 || template[suite.len()] != b'.' {
        return Err(TemplateIdentifierRefusal::OutsideSuite);
    }
    let mut index = 0;
    while index < suite.len() {
        if suite[index] != template[index] {
            return Err(TemplateIdentifierRefusal::OutsideSuite);
        }
        index += 1;
    }
    template_segment(template, suite.len() + 1)
}

/* === Private Functions === */

const fn template_segment(identifier: &[u8], start: usize) -> Result<(), TemplateIdentifierRefusal> {
    if start >= identifier.len() || !identifier[start].is_ascii_lowercase() {
        return Err(TemplateIdentifierRefusal::InvalidSegment);
    }
    let mut index = start;
    while index < identifier.len() {
        let byte = identifier[index];
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') {
            return Err(TemplateIdentifierRefusal::InvalidSegment);
        }
        index += 1;
    }
    Ok(())
}

/* === Tests === */

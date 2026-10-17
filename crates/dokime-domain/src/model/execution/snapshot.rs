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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::model::identity::DocumentId;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::TemplateId;
use crate::model::values::Value;
use crate::model::definition::{
    MaterializedParameter, ParameterDefinition, ParameterSource, TestCase, TestCaseTemplate,
    TestSuite,
};
use crate::model::document::DocumentDefinition;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("parameter {name:?}: {reason}")]
pub struct SnapshotError {
    pub name: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExecutionPluginIdentity {
    pub source: String,
    #[serde(default)]
    pub version: Option<String>,
    pub implementation: ExecutionImplementation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSnapshot {
    case: TestCase,
    template: Option<TestCaseTemplate>,
    #[serde(default)]
    parameter_definitions: Vec<ParameterDefinition>,
    plugin: Option<ExecutionPluginIdentity>,
    #[serde(default)]
    host_version: Option<String>,
    parameters: BTreeMap<String, MaterializedParameter>,
    documents: BTreeMap<DocumentId, DocumentDefinition>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ExecutionImplementation {
    Bundle { content_digest: String },
    Embedded { generation: String },
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for ExecutionSnapshot {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.case),
                std::mem::size_of_val(&self.case),
            ),
            (
                harmos::MemorySize::memory_size(&self.template),
                std::mem::size_of_val(&self.template),
            ),
            (
                harmos::MemorySize::memory_size(&self.parameter_definitions),
                std::mem::size_of_val(&self.parameter_definitions),
            ),
            (
                harmos::MemorySize::memory_size(&self.plugin),
                std::mem::size_of_val(&self.plugin),
            ),
            (
                harmos::MemorySize::memory_size(&self.host_version),
                std::mem::size_of_val(&self.host_version),
            ),
            (
                harmos::MemorySize::memory_size(&self.parameters),
                std::mem::size_of_val(&self.parameters),
            ),
            (
                harmos::MemorySize::memory_size(&self.documents),
                std::mem::size_of_val(&self.documents),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for ExecutionPluginIdentity {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.version),
                std::mem::size_of_val(&self.version),
            ),
            (
                harmos::MemorySize::memory_size(&self.implementation),
                std::mem::size_of_val(&self.implementation),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for ExecutionImplementation {
    fn memory_size(&self) -> usize {
        let value = match self {
            Self::Bundle { content_digest } => content_digest,
            Self::Embedded { generation } => generation,
        };
        std::mem::size_of::<Self>().saturating_add(value.capacity())
    }
}

impl ExecutionPluginIdentity {
    pub fn content_digest(&self) -> Option<&str> {
        match &self.implementation {
            ExecutionImplementation::Bundle { content_digest } => Some(content_digest),
            ExecutionImplementation::Embedded { .. } => None,
        }
    }
}

impl ExecutionSnapshot {
    /// Reassembles a snapshot from parts captured earlier without materializing
    /// them again; they were validated when the snapshot was first built.
    pub fn from_captured(
        case: TestCase,
        template: Option<TestCaseTemplate>,
        parameter_definitions: Vec<ParameterDefinition>,
        plugin: Option<ExecutionPluginIdentity>,
        host_version: Option<String>,
        parameters: BTreeMap<String, MaterializedParameter>,
        documents: BTreeMap<DocumentId, DocumentDefinition>,
    ) -> Self {
        Self {
            case,
            template,
            parameter_definitions,
            plugin,
            host_version,
            parameters,
            documents,
        }
    }

    pub fn materialize(
        case: &TestCase,
        suite: Option<&TestSuite>,
        documents: BTreeMap<DocumentId, DocumentDefinition>,
        plugin: Option<ExecutionPluginIdentity>,
        settings: BTreeMap<String, Value>,
    ) -> Result<Self, SnapshotError> {
        if case.suite_id.trim().is_empty() {
            return Err(SnapshotError {
                name: "suite_id".into(),
                reason: "suite registration ID is required".into(),
            });
        }

        let template = suite
            .and_then(|suite| {
                suite
                    .test_case_templates
                    .iter()
                    .find(|template| template.test_case_template_id == case.test_case_template_id)
            })
            .cloned();
        let definitions = template
            .iter()
            .flat_map(|template| template.parameters.iter())
            .map(|definition| (definition.metadata.identifier.clone(), definition.clone()))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();
        let mut parameters = BTreeMap::new();
        for (key, value) in &case.parameters {
            let identifier = Identifier::new(key).map_err(|error| SnapshotError {
                name: key.clone(),
                reason: error.to_string(),
            })?;
            let parameter = definitions
                .iter()
                .find(|definition| definition.metadata.identifier.as_str() == key)
                .and_then(|definition| {
                    crate::model::definition::materialized_parameter(
                        definition,
                        ParameterSource::Definition,
                        value.clone(),
                    )
                    .ok()
                })
                .unwrap_or_else(|| MaterializedParameter {
                    metadata: Metadata::new(identifier, key, None),
                    group_id: None,
                    source: ParameterSource::Definition,
                    value: value.clone(),
                    unit: None,
                });
            parameters.insert(key.clone(), parameter);
        }
        // Declared but absent inputs stay addressable as null. Admission rejects
        // required nulls; typed optional reads can distinguish these from typos.
        for definition in &definitions {
            let parameter = parameters
                .entry(definition.metadata.identifier.to_string())
                .or_insert_with(|| MaterializedParameter {
                    metadata: definition.metadata.clone(),
                    group_id: definition.group_id.clone(),
                    source: crate::model::definition::ParameterSource::Definition,
                    value: Value::Null,
                    unit: definition.unit.clone(),
                });
            // Preserve invalid raw case inputs for admission to reject rather
            // than hiding them behind a declaration default.
            if let Ok(resolved) = crate::model::definition::materialized_parameter(
                definition,
                parameter.source.clone(),
                parameter.value.clone(),
            ) {
                *parameter = resolved;
            }
        }
        for (key, value) in settings {
            if parameters.contains_key(&key) {
                return Err(SnapshotError {
                    name: key,
                    reason: "plugin setting collides with a case parameter".into(),
                });
            }
            let identifier = Identifier::new(&key).map_err(|error| SnapshotError {
                name: key.clone(),
                reason: error.to_string(),
            })?;
            parameters.insert(
                key.clone(),
                MaterializedParameter {
                    metadata: Metadata::definition(identifier, &key, None, ["input"]),
                    group_id: None,
                    source: ParameterSource::Runtime,
                    value,
                    unit: None,
                },
            );
        }
        Ok(Self {
            case: case.clone(),
            template,
            parameter_definitions: definitions,
            plugin,
            host_version: None,
            parameters,
            documents,
        })
    }

    pub fn with_host_version(mut self, version: Option<String>) -> Self {
        self.host_version = version;
        self
    }

    pub fn host_version(&self) -> Option<&str> {
        self.host_version.as_deref()
    }

    pub fn case(&self) -> &TestCase {
        &self.case
    }

    // An Operator Rename Changes Only The Case Label; Its Identity And Inputs Stay
    pub fn relabel_case(&mut self, label: String) {
        self.case.metadata.label = label;
    }

    pub fn suite(&self) -> Option<&Metadata> {
        self.case.suite.as_ref()
    }
    pub fn template_id(&self) -> Option<&TemplateId> {
        Some(&self.case.test_case_template_id)
    }
    pub fn template(&self) -> Option<&TestCaseTemplate> {
        self.template.as_ref()
    }
    pub fn plugin(&self) -> Option<&ExecutionPluginIdentity> {
        self.plugin.as_ref()
    }
    pub fn parameter_definitions(&self) -> &[ParameterDefinition] {
        &self.parameter_definitions
    }

    pub fn parameters(&self) -> &BTreeMap<String, MaterializedParameter> {
        &self.parameters
    }
    pub fn documents(&self) -> &BTreeMap<DocumentId, DocumentDefinition> {
        &self.documents
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

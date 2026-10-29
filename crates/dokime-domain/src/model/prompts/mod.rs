/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod checks;
mod rules;
mod validation;
pub mod wire;

pub use checks::{validate_answer_rules, validate_answers, validate_definition, validate_fields};
pub use rules::Rule;
pub use rules::validate_rules;
pub use validation::FieldValidation;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

use crate::model::definition::RuntimeCondition;
use crate::model::execution::RecordScope;
use crate::model::identity::ContractId;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::PromptId;
use crate::model::identity::SuiteId;
use crate::model::identity::TestCycleId;
use crate::model::identity::TestExecutionId;
use crate::model::records::RecordSeverity;
use crate::model::time::Timestamp;
use crate::model::values::Value;
use crate::model::values::ValueType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptDefinition {
    pub prompt_type: Identifier,
    pub schema_version: u32,
    pub archetype: PromptArchetype,
    pub title: String,
    pub message: Option<String>,

    #[serde(default)]
    pub fields: Vec<PromptField>,

    #[serde(default)]
    pub actions: Vec<PromptAction>,

    #[serde(default)]
    pub rules: Vec<Rule>,

    pub condition: Option<RuntimeCondition>,
    pub owner: Option<SuiteId>,
    pub template_identifier: Option<Identifier>,

    #[serde(default)]
    pub examples: Vec<BTreeMap<String, Value>>,

    // How urgently an operator must look: the author declares it once per
    // prompt type, on the record severity scale, and it defaults to Info.
    #[serde(default)]
    pub level: RecordSeverity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptField {
    pub metadata: Metadata,
    pub value_type: ValueType,
    pub kind: PromptFieldKind,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<Value>,

    #[serde(default)]
    pub choices: Vec<String>,

    #[serde(default)]
    pub validation: FieldValidation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptRequest {
    pub prompt_id: PromptId,
    pub prompt_type: Identifier,

    #[serde(default)]
    pub values: BTreeMap<String, Value>,

    pub scope: RecordScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptInstance {
    pub prompt_id: PromptId,
    pub definition: PromptDefinition,
    pub kind: PromptKind,
    pub allow_timeout: bool,
    pub expires_at: Option<Timestamp>,
    pub values: BTreeMap<String, Value>,
    pub scope: RecordScope,
    pub created_at: Timestamp,
    // Retained for the current historical record projection; never authored.
    pub contract_id: ContractId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptResponse {
    pub prompt_id: PromptId,
    pub action: String,
    pub responder: String,
    pub submitted_at: Timestamp,
    pub values: BTreeMap<String, Value>,
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptAction {
    pub id: String,
    pub label: String,
    pub emphasis: PromptActionEmphasis,

    #[serde(default)]
    pub requires_condition: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PromptArchetype {
    ConfirmationPrompt,
    FormPrompt,
    ChoicePrompt,
    StatusPrompt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    Confirmation,
    Form,
    Choice,
    Status,
    Verification,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptStatus {
    Pending,
    Answered,
    Cancelled,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptActionEmphasis {
    Primary,
    Secondary,
    Danger,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptFieldKind {
    Text,
    Float,
    Int,
    Choice,
    Boolean,
}

impl harmos::MemorySize for PromptInstance {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.prompt_id), std::mem::size_of_val(&self.prompt_id)),
            (harmos::MemorySize::memory_size(&self.definition), std::mem::size_of_val(&self.definition)),
            (harmos::MemorySize::memory_size(&self.expires_at), std::mem::size_of_val(&self.expires_at)),
            (harmos::MemorySize::memory_size(&self.values), std::mem::size_of_val(&self.values)),
            (harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos::MemorySize::memory_size(&self.created_at), std::mem::size_of_val(&self.created_at)),
            (harmos::MemorySize::memory_size(&self.contract_id), std::mem::size_of_val(&self.contract_id)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for PromptDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.prompt_type), std::mem::size_of_val(&self.prompt_type)),
            (harmos::MemorySize::memory_size(&self.title), std::mem::size_of_val(&self.title)),
            (harmos::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos::MemorySize::memory_size(&self.fields), std::mem::size_of_val(&self.fields)),
            (harmos::MemorySize::memory_size(&self.actions), std::mem::size_of_val(&self.actions)),
            (harmos::MemorySize::memory_size(&self.rules), std::mem::size_of_val(&self.rules)),
            (harmos::MemorySize::memory_size(&self.condition), std::mem::size_of_val(&self.condition)),
            (harmos::MemorySize::memory_size(&self.owner), std::mem::size_of_val(&self.owner)),
            (harmos::MemorySize::memory_size(&self.template_identifier), std::mem::size_of_val(&self.template_identifier)),
            (harmos::MemorySize::memory_size(&self.examples), std::mem::size_of_val(&self.examples)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for PromptField {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.default), std::mem::size_of_val(&self.default)),
            (harmos::MemorySize::memory_size(&self.choices), std::mem::size_of_val(&self.choices)),
            (harmos::MemorySize::memory_size(&self.validation), std::mem::size_of_val(&self.validation)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for PromptAction {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.id), std::mem::size_of_val(&self.id)),
            (harmos::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl PromptDefinition {
    pub fn kind(&self) -> PromptKind {
        match self.archetype {
            PromptArchetype::ConfirmationPrompt => PromptKind::Confirmation,
            PromptArchetype::FormPrompt => PromptKind::Form,
            PromptArchetype::ChoicePrompt => PromptKind::Choice,
            PromptArchetype::StatusPrompt => PromptKind::Status,
        }
    }
}

pub fn pending_stream(cycle: Option<&TestCycleId>, execution: Option<&TestExecutionId>) -> String {
    let selection = serde_json::to_vec(&(cycle, execution)).expect("prompt selection encodes");
    format!("prompts.pending:{}", Sha256::digest(selection).iter().map(|byte| format!("{byte:02x}")).collect::<String>())
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

#[allow(unused_imports)]
use crate::model::definition::wire::*;

#[allow(unused_imports)]
use crate::model::execution::wire::*;
use crate::model::execution::wire_cursor::Cursor;
use crate::model::identity::wire::Metadata;

#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::values::wire::Value;

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPromptDefinitionsReply {
    pub definitions: Vec<PromptDefinitionIdentity>,
    pub next_after_prompt_type: Option<String>,
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptDefinitionIdentity {
    pub prompt_type: String,
    pub schema_version: u32,
    pub title: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPendingPromptsReply {
    pub prompts: Vec<PendingPromptIdentity>,
    pub next_after_prompt_id: Option<String>,
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingPromptIdentity {
    pub prompt_id: String,
    pub definition: Option<PromptDefinitionIdentity>,
    pub scope: Option<EvidenceScope>,
    pub created_at: Option<Timestamp>,
}

// The admitted definition belongs to this request even if the current catalog
// changes. A missing response means unanswered; it does not waive condition,
// action or field checks performed authoritatively on Respond.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptView {
    pub prompt: Option<PromptInstance>,
    pub response: Option<PromptResponse>,
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptInstance {
    pub prompt_id: String,
    pub definition: Option<PromptDefinition>,
    pub kind: PromptKind,
    pub values: Option<Value>,
    pub scope: Option<EvidenceScope>,
    pub created_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptDefinition {
    pub prompt_type: String,
    pub schema_version: u32,
    pub archetype: PromptArchetype,
    pub title: String,
    pub message: Option<String>,
    pub fields: Vec<PromptField>,
    pub actions: Vec<PromptAction>,
    pub rules: Vec<AnswerRule>,
    pub condition: Option<EvidenceCondition>,
    pub owner_suite_id: Option<String>,
    pub template_identifier: Option<String>,
    pub examples: Vec<Value>,

    #[row(description = "The declared level on the record severity scale; Unspecified reads as Info.")]
    pub level: Severity,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptField {
    pub metadata: Option<Metadata>,
    pub value_type: ValueType,
    pub kind: PromptFieldKind,
    pub multiple: bool,
    pub required: bool,
    pub default_value: Option<Value>,
    pub choices: Vec<String>,
    pub validation: Option<PromptFieldValidation>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptFieldValidation {
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub pattern: Option<String>,
    pub minimum: Option<Value>,
    pub maximum: Option<Value>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptAction {
    pub id: String,
    pub label: String,
    pub emphasis: PromptActionEmphasis,
    pub requires_condition: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerRule {
    pub predicate: Option<AnswerPredicate>,
    pub message: String,
}

// Predicates over answer fields. Calls and structured literals are not admitted
// in answer rules. Validation remains authoritative on the backend.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
//
// The tree is flattened into its nodes: `nodes[0]` is the root and a node
// names its operands by a larger index in `nodes`.
pub struct AnswerPredicate {
    pub nodes: Vec<AnswerPredicateNode>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerPredicateNode {
    pub kind: Option<AnswerPredicateKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerBinaryPredicate {
    pub op: AnswerOperator,

    #[row(description = "Operand index.")]
    pub left: u32,

    #[row(description = "Operand index.")]
    pub right: u32,
}

// A condition tree flattened into its nodes: `nodes[0]` is the root and a
// node names its operands by a larger index in `nodes`.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceCondition {
    pub nodes: Vec<EvidenceConditionNode>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceConditionNode {
    pub kind: Option<EvidenceConditionKind>,
}

// Operand indices into the enclosing condition's nodes.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceConditions {
    pub conditions: Vec<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordMatchCondition {
    pub record_type: String,
    pub match_fields: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryMatchCondition {
    pub stream: String,
    pub field: String,
    pub op: TelemetryMatchOperator,
    pub target: Option<Value>,
    pub tolerance: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptResponseReceipt {
    pub command_id: String,
    pub prompt_id: String,
    pub committed_revision: u64,
    pub accepted_at: Option<Timestamp>,
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptResponse {
    pub prompt_id: String,
    pub action: String,
    pub responder: String,
    pub submitted_at: Option<Timestamp>,
    pub values: Option<Value>,
    pub accepted: bool,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptArchetype {
    Unspecified,
    Confirmation,
    Form,
    Choice,
    Status,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptKind {
    Unspecified,
    Confirmation,
    Form,
    Choice,
    Status,
    Verification,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptFieldKind {
    Unspecified,
    Text,
    Float,
    Int,
    Choice,
    Boolean,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptActionEmphasis {
    Unspecified,
    Primary,
    Secondary,
    Danger,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerOperator {
    Unspecified,
    And,
    Or,
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryMatchOperator {
    Unspecified,
    Exists,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Within,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AnswerPredicateKind {
    Literal(Value),
    Field(String),
    // Operand index.
    Not(u32),
    Binary(AnswerBinaryPredicate),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceConditionKind {
    Record(RecordMatchCondition),
    Telemetry(TelemetryMatchCondition),
    All(EvidenceConditions),
    Any(EvidenceConditions),
}

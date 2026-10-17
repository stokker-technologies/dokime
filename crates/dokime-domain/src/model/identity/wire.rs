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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::values::wire::Value;
use crate::model::values::wire::ValueError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Authored identity and description of a Dokime object.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    // In key order; keys are unique.
    #[row(description = "In key order; keys are unique.")]
    pub attributes: Vec<MetadataAttribute>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataAttribute {
    pub key: String,
    pub value: Option<Value>,
}

// Protobuf well-known timestamp shape: seconds and nanoseconds since the Unix epoch.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

// Backend-owned evidence scope. Identifiers are not client filesystem paths.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceScope {
    pub session_id: Option<String>,
    pub cycle_id: Option<String>,
    pub case_id: Option<String>,
    pub execution_id: Option<String>,
    pub step_id: Option<String>,
    pub criterion_id: Option<String>,
    pub template_id: Option<String>,
    pub suite_id: Option<String>,
    pub origin: Option<EvidenceOrigin>,
}

// Execution admission attribution; producer occurrence and capture times stay separate.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceOrigin {
    pub kind: Option<EvidenceOriginKind>,
}

// A current or historical traceability group. The snapshot retains the
// authored entry metadata, value and provenance source; consumers must not
// reconstruct provenance from the display label.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilitySnapshot {
    pub traceability_id: String,
    pub scope: Option<EvidenceScope>,
    pub created_at: Option<Timestamp>,
    pub entries: Vec<TraceabilityEntry>,
    // What this group is, when something said so. A template states its
    // traceability as labelled sections and a section is a group, so the label
    // travels with it; the groups the runtime derives on its own carry none.
    #[row(
        description = "What this group is, when something said so. A template states its traceability as labelled sections and a section is a group, so the label travels with it; the groups the runtime derives on its own carry none."
    )]
    pub metadata: Option<Metadata>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityEntry {
    pub metadata: Option<Metadata>,
    pub value: Option<Value>,
    pub source: Option<String>,
    // The unit the value is stated in, for a value that has one.
    #[row(description = "The unit the value is stated in, for a value that has one.")]
    pub unit: Option<String>,
}

// The whole story behind one failure: the message an operator reads, the cause
// chain beneath it, where it was captured, the spans it happened inside, and
// the frames a maintainer needs. Redacted before it is stored, so no absolute
// home directory or account name ever reaches a consumer.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub chain: Vec<String>,
    pub location: String,
    pub span_trace: Vec<String>,
    pub backtrace: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("metadata identifier is invalid")]
    Identifier,
    #[error("metadata attribute is missing or repeated")]
    Attribute,
    #[error("metadata timestamp is out of range")]
    Timestamp,
    #[error(transparent)]
    Value(#[from] ValueError),
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Unspecified,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceOriginKind {
    Default(bool),
    Initialization(bool),
    StepAttemptId(String),
    Cleanup(bool),
}

/* === Traits === */

/* === Implementations === */
impl From<&crate::model::identity::Metadata> for Metadata {
    fn from(metadata: &crate::model::identity::Metadata) -> Self {
        Self {
            identifier: metadata.identifier.to_string(),
            label: metadata.label.clone(),
            description: metadata.description.clone(),
            tags: metadata.tags.clone(),
            attributes: metadata
                .attributes
                .iter()
                .map(|(key, value)| MetadataAttribute {
                    key: key.clone(),
                    value: Some(Value::from(value)),
                })
                .collect(),
            created_at: metadata.created_at.map(Timestamp::from),
            updated_at: metadata.updated_at.map(Timestamp::from),
        }
    }
}

impl TryFrom<&Metadata> for crate::model::identity::Metadata {
    type Error = MetadataError;

    fn try_from(metadata: &Metadata) -> Result<Self, Self::Error> {
        let mut attributes = std::collections::BTreeMap::new();
        for attribute in &metadata.attributes {
            let value = attribute
                .value
                .as_ref()
                .ok_or(MetadataError::Attribute)?
                .to_core()?;
            if attributes.insert(attribute.key.clone(), value).is_some() {
                return Err(MetadataError::Attribute);
            }
        }
        let time = |value: Option<Timestamp>| {
            value
                .map(|value| value.to_datetime().map_err(|_| MetadataError::Timestamp))
                .transpose()
        };
        Ok(Self {
            identifier: crate::model::identity::Identifier::new(&metadata.identifier)
                .map_err(|_| MetadataError::Identifier)?,
            label: metadata.label.clone(),
            description: metadata.description.clone(),
            tags: metadata.tags.clone(),
            attributes,
            created_at: time(metadata.created_at)?,
            updated_at: time(metadata.updated_at)?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

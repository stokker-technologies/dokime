/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Product Objects

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_domain::{Error, Identified, Identifier, Result};
use harmos_storage::{ObjectCatalog, ObjectDefinition, ObjectRetention, StoredObject};

// Local Imports
use super::bindings::Protobuf;

/* === Constants === */

/* === Macros === */

macro_rules! object {
    ($type:ident, $id:literal, [$($owner:literal),+]) => {
        object!($type, $id, [$($owner),+], false);
    };
    ($type:ident, $id:literal, [$($owner:literal),+], $exact:literal) => {
        impl Identified for $type {
            const ID: &'static str = $id;
        }
        impl StoredObject for $type {
            type Key = str;
            const RETENTION: ObjectRetention = ObjectRetention::All;
            fn key(&self) -> &str {
                &self.key
            }
            fn scope(&self) -> Option<&Identifier> {
                Some(&self.scope)
            }
            fn validate(&self) -> Result<()> {
                let allowed = [$($owner),+];
                let valid = self.scope.as_str().split_once('/').is_some_and(|(kind, key)| !key.is_empty() && allowed.contains(&kind));
                let exact = !$exact || self.scope.as_str() == format!("{}/{}", allowed[0], self.key);
                if !valid || !exact {
                    return Err(Error::InvalidInput { field: "product object scope", reason: "select the object's exact durable owner" });
                }
                Ok(())
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Root {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Cycle {
    pub key: String,
    pub scope: Identifier,
    pub metadata: Protobuf,
    pub sealed: bool,
    pub outcomes: Outcomes,
}

/// Attributed Seal Outcomes; Membership Is Always Read From Surviving Bindings.
#[derive(harmos_domain::Message, Clone, Debug, Default, PartialEq)]
pub struct Outcomes {
    pub passed: u64,
    pub failed: u64,
    pub blocked: u64,
    pub skipped: u64,
    pub declared: u64,
    pub not_evaluated: u64,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Execution {
    pub key: String,
    pub scope: Identifier,
    pub case: String,
    pub cycle: String,
    pub origin: Protobuf,
    pub catalog: Vec<u8>,
    pub state: State,
    pub outcome: Option<Protobuf>,
    pub termination: i32,
    pub seal: Option<Seal>,
    /// Complete, Bounded Derived Object Inventory, Frozen Before Admission.
    pub previews: Vec<String>,
}

/// C Is an Issued Cut, Not a Predicted Publication Position.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Seal {
    pub evidence_cut: Vec<u8>,
    pub retained_snapshot: Vec<u8>,
    pub digest: Vec<u8>,
    pub receipt_ns: i64,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Parameters {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Steps {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Progress {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

/// Frozen Stream Binding; the Library Schema Owns the Arrow Sample Payload.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Stream {
    pub key: String,
    pub scope: Identifier,
    pub declaration: String,
    pub schema: String,
    pub version: u32,
    pub message: String,
    pub descriptors: Vec<u8>,
    pub closed: bool,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Prompt {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Document {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct PreparedReport {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
    pub evidence_cut: Vec<u8>,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Report {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
    pub evidence_cut: Vec<u8>,
    pub file: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Artifact {
    pub key: String,
    pub scope: Identifier,
    pub value: Protobuf,
    pub file: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct ExecutionFile {
    pub key: String,
    pub scope: Identifier,
    pub logical_path: String,
    pub file: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Preview {
    pub key: String,
    pub scope: Identifier,
    pub source_cut: Vec<u8>,
    pub kind: String,
    pub selection: Vec<u8>,
    pub payload: Vec<u8>,
}

/* === Enums === */

#[derive(harmos_domain::Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    Closed,
    Sealed,
}

/* === Traits === */

/* === Implementations === */

object!(Root, "dokime.root", ["root"], true);
object!(Cycle, "dokime.cycle", ["cycle"], true);
object!(Execution, "dokime.execution", ["execution"], true);
object!(Parameters, "dokime.parameters", ["execution"]);
object!(Steps, "dokime.steps", ["execution"]);
object!(Progress, "dokime.progress", ["execution"]);
object!(Stream, "dokime.stream", ["execution"]);
object!(Prompt, "dokime.prompt", ["execution"]);
object!(Document, "dokime.document", ["execution"]);
object!(PreparedReport, "dokime.prepared-report", ["execution", "cycle"]);
object!(Report, "dokime.report", ["execution", "cycle"]);
object!(Artifact, "dokime.artifact", ["execution", "cycle"]);
object!(ExecutionFile, "dokime.execution-file", ["execution"]);
object!(Preview, "dokime.preview", ["execution"]);

/* === Public Functions === */

pub fn catalog() -> ObjectCatalog {
    ObjectCatalog::new()
        .entry(ObjectDefinition::of::<Root>())
        .entry(ObjectDefinition::of::<Cycle>())
        .entry(ObjectDefinition::of::<Execution>())
        .entry(ObjectDefinition::of::<Parameters>())
        .entry(ObjectDefinition::of::<Steps>())
        .entry(ObjectDefinition::of::<Progress>())
        .entry(ObjectDefinition::of::<Stream>())
        .entry(ObjectDefinition::of::<Prompt>())
        .entry(ObjectDefinition::of::<Document>())
        .entry(ObjectDefinition::of::<PreparedReport>())
        .entry(ObjectDefinition::of::<Report>())
        .entry(ObjectDefinition::of::<Artifact>())
        .entry(ObjectDefinition::of::<ExecutionFile>())
        .entry(ObjectDefinition::of::<Preview>())
}

/* === Private Functions === */

/* === Tests === */

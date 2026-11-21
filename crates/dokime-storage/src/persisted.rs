/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Scoped Product Objects

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_domain::{Identified, Identifier};
use harmos_storage::{ObjectCatalog, ObjectDefinition, ObjectRetention, StoredObject};

// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Product Protobuf Bytes With Their Explicit Durable Owner.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Administration {
    pub key: String,
    pub scope: Identifier,
    pub payload: Vec<u8>,
}

/// Frozen Planned Case, Owned Independently of Its Registration Replay.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Case {
    pub key: String,
    pub scope: Identifier,
    pub cycle: String,
    pub payload: Vec<u8>,
}

/// Original Registration Bytes and Native Mutation Identity for Durable Replay.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Plan {
    pub key: String,
    pub scope: Identifier,
    pub plan: Vec<u8>,
    pub cases: Vec<Case>,
    pub window: Identifier,
    pub mutation: Identifier,
}

/// Execution Command State and Its Original Admission for Receipt Replay.
/// Both Payloads Decode as `dokime.host.v1.executions.ExecutionCommandReceipt`.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Command {
    pub key: String,
    pub scope: Identifier,
    pub current: Vec<u8>,
    pub original: Vec<u8>,
    pub window: Identifier,
    pub mutation: Identifier,
}

/* === Enums === */

/// Exact Opaque Owners; Relationships Are Product Data, Never Scope Prefixes.
pub enum Scope<'a> {
    Root(&'a str),
    Case(&'a str),
    Cycle(&'a str),
    CycleSeal(&'a str),
    Execution(&'a str),
}

/* === Traits === */

/* === Implementations === */

impl Scope<'_> {
    pub fn identifier(&self) -> Result<Identifier> {
        let (kind, key) = match self {
            Self::Root(key) => ("root", key),
            Self::Case(key) => ("case", key),
            Self::Cycle(key) => ("cycle", key),
            Self::CycleSeal(key) => ("cycle-seal", key),
            Self::Execution(key) => ("execution", key),
        };
        if key.is_empty() {
            return Err(Error::Invalid("Resolve Storage Scope: Select a Nonempty Product Owner".into()));
        }
        Identifier::new(format!("{kind}/{key}"))
            .map_err(|source| Error::Invalid(format!("Resolve Storage Scope: Select a Valid Product Owner: {source}")))
    }
}

impl Identified for Case {
    const ID: &'static str = "dokime.case";
}

impl Identified for Administration {
    const ID: &'static str = "dokime.administration";
}

impl StoredObject for Administration {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::All;

    fn key(&self) -> &str {
        &self.key
    }

    fn scope(&self) -> Option<&Identifier> {
        Some(&self.scope)
    }
}

impl StoredObject for Case {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::All;

    fn key(&self) -> &str {
        &self.key
    }

    fn scope(&self) -> Option<&Identifier> {
        Some(&self.scope)
    }
}

impl Identified for Plan {
    const ID: &'static str = "dokime.plan";
}

impl StoredObject for Plan {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::All;

    fn key(&self) -> &str {
        &self.key
    }

    fn scope(&self) -> Option<&Identifier> {
        Some(&self.scope)
    }
}

impl Identified for Command {
    const ID: &'static str = "dokime.command";
}

impl StoredObject for Command {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::All;

    fn key(&self) -> &str {
        &self.key
    }

    fn scope(&self) -> Option<&Identifier> {
        Some(&self.scope)
    }
}

/* === Public Functions === */

pub fn catalog() -> ObjectCatalog {
    ObjectCatalog::new()
        .entry(ObjectDefinition::of::<Administration>())
        .entry(ObjectDefinition::of::<Case>())
        .entry(ObjectDefinition::of::<Plan>())
        .entry(ObjectDefinition::of::<Command>())
}

/* === Private Functions === */

/* === Tests === */

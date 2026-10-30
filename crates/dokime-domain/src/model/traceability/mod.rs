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
// Third-Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::identity::TraceabilityId;
use crate::model::time::Timestamp;
use crate::model::values::Value;

/* === Constants === */

const TRACEABILITY_SOURCE_PREFIX: &str = "traceability.";
pub const TEST_DEFINITION_TRACEABILITY: DerivedTraceabilityGroup = DerivedTraceabilityGroup {
    source: "traceability.test_definition",
    label: "Test Definition",
};
pub const TEST_EXECUTION_TRACEABILITY: DerivedTraceabilityGroup = DerivedTraceabilityGroup {
    source: "traceability.test_execution",
    label: "Test Execution",
};
pub const LEASES_TRACEABILITY: DerivedTraceabilityGroup = DerivedTraceabilityGroup {
    source: "traceability.leases",
    label: "Leases",
};
pub const HOST_TRACEABILITY: DerivedTraceabilityGroup = DerivedTraceabilityGroup {
    source: "traceability.host",
    label: "Host",
};
pub const DERIVED_TRACEABILITY_GROUPS: &[DerivedTraceabilityGroup] =
    &[TEST_DEFINITION_TRACEABILITY, TEST_EXECUTION_TRACEABILITY, LEASES_TRACEABILITY, HOST_TRACEABILITY];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilitySection {
    pub identifier: Identifier,
    pub label: String,

    #[serde(default)]
    pub rows: Vec<TraceabilityRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityRow {
    pub identifier: Identifier,
    pub label: String,
    pub value: Value,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivedTraceabilityGroup {
    pub source: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilitySnapshot {
    pub traceability_id: TraceabilityId,
    pub scope: RecordScope,
    pub created_at: Timestamp,

    #[serde(default)]
    pub metadata: Option<Metadata>,

    pub entries: Vec<TraceabilityEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityEntry {
    pub metadata: Metadata,
    pub value: Value,
    pub source: Option<String>,

    #[serde(default)]
    pub unit: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for TraceabilitySection {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.identifier), std::mem::size_of_val(&self.identifier)),
            (harmos::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos::MemorySize::memory_size(&self.rows), std::mem::size_of_val(&self.rows)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for TraceabilityRow {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.identifier), std::mem::size_of_val(&self.identifier)),
            (harmos::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos::MemorySize::memory_size(&self.source), std::mem::size_of_val(&self.source)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl TraceabilitySection {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            identifier: Identifier::declared(identifier, "traceability section"),
            label: label.into(),
            rows: Vec::new(),
        }
    }

    // Sections grow by statements, as a document `Section` does; a row is
    // built in one expression and appended.
    pub fn row(&mut self, row: TraceabilityRow) {
        self.rows.push(row);
    }

    pub fn rows(&mut self, rows: impl IntoIterator<Item = TraceabilityRow>) {
        self.rows.extend(rows);
    }
}

impl TraceabilityRow {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, value: Value) -> Self {
        Self {
            identifier: Identifier::declared(identifier, "traceability row"),
            label: label.into(),
            value,
            unit: None,
            source: None,
        }
    }

    pub fn text(identifier: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(identifier, label, Value::Text(value.into()))
    }

    pub fn number(identifier: impl Into<String>, label: impl Into<String>, value: f64) -> Self {
        Self::new(identifier, label, Value::Number(value))
    }

    pub fn integer(identifier: impl Into<String>, label: impl Into<String>, value: i64) -> Self {
        Self::new(identifier, label, Value::Integer(value))
    }

    pub fn boolean(identifier: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self::new(identifier, label, Value::Bool(value))
    }

    #[must_use]
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl harmos::MemorySize for TraceabilitySnapshot {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.traceability_id), std::mem::size_of_val(&self.traceability_id)),
            (harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos::MemorySize::memory_size(&self.created_at), std::mem::size_of_val(&self.created_at)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.entries), std::mem::size_of_val(&self.entries)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for TraceabilityEntry {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos::MemorySize::memory_size(&self.source), std::mem::size_of_val(&self.source)),
            (harmos::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl DerivedTraceabilityGroup {
    pub fn identifier(&self) -> &'static str {
        self.source.strip_prefix(TRACEABILITY_SOURCE_PREFIX).expect("a derived traceability source sits in the host's namespace")
    }
}

/* === Public Functions === */

/// Validate A Step-Authored Section Without Claiming A Host-Derived Group.
pub fn validate_section(section: &TraceabilitySection) -> Result<(), String> {
    if let Some(derived) = derived_traceability_group_named(section.identifier.as_str()) {
        return Err(format!("section \"{}\" claims the host-derived \"{}\" group", section.identifier, derived.label));
    }
    let mut rows = std::collections::BTreeSet::new();
    for row in &section.rows {
        if !rows.insert(&row.identifier) {
            return Err(format!("row \"{}\" occurs twice in section \"{}\"", row.identifier, section.identifier));
        }
    }
    Ok(())
}

/// Replace A Section In Place Or Append It On First Contribution.
pub fn upsert_section(sections: &mut Vec<TraceabilitySection>, section: TraceabilitySection) {
    match sections.iter_mut().find(|existing| existing.identifier == section.identifier) {
        Some(existing) => *existing = section,
        None => sections.push(section),
    }
}

pub fn derived_traceability_group(source: &str) -> Option<DerivedTraceabilityGroup> {
    let name = source.strip_prefix(TRACEABILITY_SOURCE_PREFIX)?;
    let name = name.split('.').next()?;
    DERIVED_TRACEABILITY_GROUPS.iter().copied().find(|group| group.identifier() == name)
}

pub fn derived_traceability_group_named(identifier: &str) -> Option<DerivedTraceabilityGroup> {
    DERIVED_TRACEABILITY_GROUPS.iter().copied().find(|group| group.identifier() == identifier)
}

/* === Private Functions === */

/* === Tests === */

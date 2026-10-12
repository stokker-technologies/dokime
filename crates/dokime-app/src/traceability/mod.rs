/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod projection;

/* === Re-Exports === */
pub use projection::{
    TraceabilityPageBudget, bounded_host_traceability_snapshot_from_sample, execution_traceability,
    host_traceability_snapshot_from_samples, lease_traceability_snapshot,
    replace_snapshots_by_source, snapshots_from_parameter_scopes, snapshots_from_parameters,
    snapshots_from_state, stated_snapshots,
};
pub(crate) use projection::{bounded_snapshots_from_state, host_traceability_page_candidate};
pub(crate) use projection::{
    resource_traceability, scope_discriminator, snapshots_from_execution_parameters,
    stated_execution_memory, stated_execution_snapshots,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use dokime::core::{Metadata, RecordScope, Timestamp, TraceabilityId, Value};

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

pub const DERIVED_TRACEABILITY_GROUPS: &[DerivedTraceabilityGroup] = &[
    TEST_DEFINITION_TRACEABILITY,
    TEST_EXECUTION_TRACEABILITY,
    LEASES_TRACEABILITY,
    HOST_TRACEABILITY,
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
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
impl harmos::MemorySize for TraceabilitySnapshot {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.traceability_id),
                std::mem::size_of_val(&self.traceability_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
            ),
            (
                harmos::MemorySize::memory_size(&self.created_at),
                std::mem::size_of_val(&self.created_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.entries),
                std::mem::size_of_val(&self.entries),
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

impl harmos::MemorySize for TraceabilityEntry {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.value),
                std::mem::size_of_val(&self.value),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.unit),
                std::mem::size_of_val(&self.unit),
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

impl DerivedTraceabilityGroup {
    pub fn identifier(&self) -> &'static str {
        self.source
            .strip_prefix(TRACEABILITY_SOURCE_PREFIX)
            .expect("a derived traceability source sits in the host's namespace")
    }
}

/* === Public Functions === */
/* === Functions === */

pub fn derived_traceability_group(source: &str) -> Option<DerivedTraceabilityGroup> {
    let name = source.strip_prefix(TRACEABILITY_SOURCE_PREFIX)?;
    let name = name.split('.').next()?;
    DERIVED_TRACEABILITY_GROUPS
        .iter()
        .copied()
        .find(|group| group.identifier() == name)
}

pub fn derived_traceability_group_named(identifier: &str) -> Option<DerivedTraceabilityGroup> {
    DERIVED_TRACEABILITY_GROUPS
        .iter()
        .copied()
        .find(|group| group.identifier() == identifier)
}

/* === Private Functions === */

/* === Tests === */

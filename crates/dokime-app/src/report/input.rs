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
use super::value::Value;
use crate::document::capture::Capture;
use crate::execution::CycleView;
use crate::records::RecordEntry;
use crate::traceability::TraceabilitySnapshot;
use dokime::core::{CaseId, DocumentId, RecordScope, SessionId, TestExecutionId, Timestamp};
use dokime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CycleReportInput {
    pub session_id: SessionId,
    pub cycle: CycleView,
    #[serde(default)]
    pub documents: BTreeMap<DocumentId, Capture>,
    #[serde(default)]
    pub plots: BTreeMap<String, Value>,
    #[serde(default)]
    pub case_documents: Vec<ReportCaseDocument>,
    pub records: Vec<RecordEntry>,
    pub telemetry: Vec<TelemetrySample>,
    pub traceability: Vec<TraceabilitySnapshot>,
    #[serde(default)]
    pub artifact_links: Vec<ReportArtifactLink>,
    #[serde(default)]
    pub appearance: Option<Value>,
    pub generated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportCaseDocument {
    pub test_case_id: CaseId,
    pub test_execution_id: Option<TestExecutionId>,
    #[serde(default)]
    pub documents: BTreeMap<DocumentId, Capture>,
    #[serde(default)]
    pub plots: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportArtifactLink {
    pub artifact_id: String,
    pub label: String,
    pub href: String,
    pub path: String,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub group_label: Option<String>,
    #[serde(default)]
    pub scope: Option<RecordScope>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for CycleReportInput {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.session_id),
                std::mem::size_of_val(&self.session_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.cycle),
                std::mem::size_of_val(&self.cycle),
            ),
            (
                harmos::MemorySize::memory_size(&self.documents),
                std::mem::size_of_val(&self.documents),
            ),
            (
                json_map_memory(&self.plots),
                std::mem::size_of_val(&self.plots),
            ),
            (
                harmos::MemorySize::memory_size(&self.case_documents),
                std::mem::size_of_val(&self.case_documents),
            ),
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
            (
                harmos::MemorySize::memory_size(&self.telemetry),
                std::mem::size_of_val(&self.telemetry),
            ),
            (
                harmos::MemorySize::memory_size(&self.traceability),
                std::mem::size_of_val(&self.traceability),
            ),
            (
                harmos::MemorySize::memory_size(&self.artifact_links),
                std::mem::size_of_val(&self.artifact_links),
            ),
            (
                json_optional_memory(&self.appearance),
                std::mem::size_of_val(&self.appearance),
            ),
            (
                harmos::MemorySize::memory_size(&self.generated_at),
                std::mem::size_of_val(&self.generated_at),
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

impl harmos::MemorySize for ReportCaseDocument {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.test_case_id),
                std::mem::size_of_val(&self.test_case_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_execution_id),
                std::mem::size_of_val(&self.test_execution_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.documents),
                std::mem::size_of_val(&self.documents),
            ),
            (
                json_map_memory(&self.plots),
                std::mem::size_of_val(&self.plots),
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

impl harmos::MemorySize for ReportArtifactLink {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.artifact_id),
                std::mem::size_of_val(&self.artifact_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.label),
                std::mem::size_of_val(&self.label),
            ),
            (
                harmos::MemorySize::memory_size(&self.href),
                std::mem::size_of_val(&self.href),
            ),
            (
                harmos::MemorySize::memory_size(&self.path),
                std::mem::size_of_val(&self.path),
            ),
            (
                harmos::MemorySize::memory_size(&self.media_type),
                std::mem::size_of_val(&self.media_type),
            ),
            (
                harmos::MemorySize::memory_size(&self.description),
                std::mem::size_of_val(&self.description),
            ),
            (
                harmos::MemorySize::memory_size(&self.group_id),
                std::mem::size_of_val(&self.group_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.group_label),
                std::mem::size_of_val(&self.group_label),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
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

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn json_memory(value: &Value, depth: usize) -> usize {
    value.memory_at(depth)
}

fn json_optional_memory(value: &Option<Value>) -> usize {
    let Some(value) = value else {
        return std::mem::size_of::<Option<Value>>();
    };
    let owned = json_memory(value, 0);
    if owned == usize::MAX {
        return usize::MAX;
    }
    std::mem::size_of::<Option<Value>>()
        .saturating_add(owned.saturating_sub(std::mem::size_of::<Value>()))
}

fn json_map_memory(values: &BTreeMap<String, Value>) -> usize {
    values.iter().fold(
        harmos::btree_map_memory::<String, Value>(values.len()),
        |bytes, (key, value)| {
            let owned = json_memory(value, 0);
            if owned == usize::MAX {
                usize::MAX
            } else {
                bytes
                    .saturating_add(key.capacity())
                    .saturating_add(owned.saturating_sub(std::mem::size_of::<Value>()))
            }
        },
    )
}

/* === Tests === */

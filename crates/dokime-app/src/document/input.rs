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
pub(crate) use super::context::Context;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};
// Local Imports
use crate::artifacts::ArtifactRecord;
use crate::core::{RecordId, RecordScope};
use crate::document::definition::DocumentDefinition;
use crate::records::RecordEntry;
use crate::runtime::storage::record_capture::CapturedRecord;
use crate::telemetry::TelemetrySample;
use dokime::execution::saved::{ExecutionOutput, ExecutionOutputContent};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentResolveInput {
    pub definition: DocumentDefinition,
    pub selection: DocumentSelection,
    pub data: DocumentDataInput,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSelection {
    pub scope: RecordScope,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentDataInput {
    pub records: Vec<RecordEntry>,
    pub telemetry: Vec<TelemetrySample>,
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Serialize)]
pub(crate) struct BorrowedInput<'a> {
    pub definition: &'a DocumentDefinition,
    pub selection: &'a DocumentSelection,
    pub data: BorrowedData<'a>,
}

#[derive(Serialize)]
pub(crate) struct BorrowedData<'a> {
    pub context: Context<'a>,
    pub records: Records<'a>,
    pub telemetry: &'a [TelemetrySample],
    pub artifacts: Artifacts<'a>,
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct RecordView<'a> {
    pub entry: &'a RecordEntry,
    pub scope: Option<&'a RecordScope>,
}

/* === Enums === */
#[derive(Clone, Copy)]
pub(crate) enum Records<'a> {
    Native(&'a [RecordEntry]),
    Captured {
        rows: &'a [CapturedRecord],
        scopes: &'a BTreeMap<RecordId, RecordScope>,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum Artifacts<'a> {
    Native(&'a [ArtifactRecord]),
    Outputs(&'a [ExecutionOutput]),
}

/* === Traits === */

/* === Implementations === */
impl DocumentResolveInput {
    pub(crate) fn borrowed(&self) -> BorrowedInput<'_> {
        BorrowedInput {
            definition: &self.definition,
            selection: &self.selection,
            data: BorrowedData {
                context: Context::default(),
                records: Records::Native(&self.data.records),
                telemetry: &self.data.telemetry,
                artifacts: Artifacts::Native(&self.data.artifacts),
            },
        }
    }
}

impl Records<'_> {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Native(rows) => rows.len(),
            Self::Captured { rows, .. } => rows.len(),
        }
    }
}

impl<'a> Records<'a> {
    pub(super) fn iter(self) -> impl DoubleEndedIterator<Item = RecordView<'a>> + Clone {
        let (native, captured, scopes) = match self {
            Self::Native(rows) => (rows, &[][..], None),
            Self::Captured { rows, scopes } => (&[][..], rows, Some(scopes)),
        };
        native
            .iter()
            .chain(captured.iter().map(|row| &**row))
            .map(move |entry| RecordView {
                entry,
                scope: scopes
                    .and_then(|scopes| scopes.get(&entry.record_id))
                    .or(entry.scope.as_ref()),
            })
    }
}

impl Default for Records<'_> {
    fn default() -> Self {
        Self::Native(&[])
    }
}

impl Serialize for Records<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.len()))?;
        for row in self.iter() {
            sequence.serialize_element(&row)?;
        }
        sequence.end()
    }
}

impl<'a> Artifacts<'a> {
    pub(super) fn len(self) -> usize {
        self.iter().count()
    }

    pub(super) fn iter(self) -> impl DoubleEndedIterator<Item = &'a ArtifactRecord> + Clone {
        let (native, outputs) = match self {
            Self::Native(rows) => (rows, &[][..]),
            Self::Outputs(outputs) => (&[][..], outputs),
        };
        native
            .iter()
            .chain(outputs.iter().filter_map(|output| match &output.content {
                ExecutionOutputContent::Artifact { record, .. } => Some(record.as_ref()),
                _ => None,
            }))
    }
}

impl Default for Artifacts<'_> {
    fn default() -> Self {
        Self::Native(&[])
    }
}

impl Serialize for Artifacts<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(None)?;
        for row in self.iter() {
            sequence.serialize_element(row)?;
        }
        sequence.end()
    }
}

impl harmos::MemorySize for BorrowedInput<'_> {
    fn memory_size(&self) -> usize {
        use harmos::MemorySize;

        // Count Reachable Native Values Without Claiming Their Existing Owners.
        let fields = [
            (
                self.definition.memory_size(),
                std::mem::size_of::<DocumentDefinition>(),
            ),
            (
                self.selection.memory_size(),
                std::mem::size_of::<DocumentSelection>(),
            ),
        ];
        let bytes = fields.into_iter().fold(
            std::mem::size_of::<DocumentResolveInput>(),
            |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            },
        );
        self.data
            .records
            .iter()
            .map(|row| {
                row.entry
                    .memory_size()
                    .saturating_add(row.scope.map_or(0, MemorySize::memory_size))
            })
            .chain(self.data.telemetry.iter().map(MemorySize::memory_size))
            .chain(self.data.artifacts.iter().map(MemorySize::memory_size))
            .fold(bytes, usize::saturating_add)
            .saturating_add(self.data.context.memory_size())
    }
}

impl harmos::MemorySize for DocumentResolveInput {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.definition),
                std::mem::size_of_val(&self.definition),
            ),
            (
                harmos::MemorySize::memory_size(&self.selection),
                std::mem::size_of_val(&self.selection),
            ),
            (
                harmos::MemorySize::memory_size(&self.data),
                std::mem::size_of_val(&self.data),
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

impl harmos::MemorySize for DocumentDataInput {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
            (
                harmos::MemorySize::memory_size(&self.telemetry),
                std::mem::size_of_val(&self.telemetry),
            ),
            (
                harmos::MemorySize::memory_size(&self.artifacts),
                std::mem::size_of_val(&self.artifacts),
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

impl harmos::MemorySize for DocumentSelection {
    fn memory_size(&self) -> usize {
        let fields = [(
            harmos::MemorySize::memory_size(&self.scope),
            std::mem::size_of_val(&self.scope),
        )];
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

impl DocumentSelection {
    pub(crate) fn foreign_execution_scope(&self, scope: Option<&RecordScope>) -> bool {
        self.scope.test_execution_id.is_some()
            && scope.is_some_and(|scope| {
                scope.test_execution_id == self.scope.test_execution_id
                    && (scope
                        .test_cycle_id
                        .as_ref()
                        .is_some_and(|id| self.scope.test_cycle_id.as_ref() != Some(id))
                        || scope
                            .test_case_id
                            .as_ref()
                            .is_some_and(|id| self.scope.test_case_id.as_ref() != Some(id)))
            })
    }

    pub(crate) fn evidence_scope(&self) -> crate::evidence::ScopeSelection {
        crate::evidence::ScopeSelection {
            test_cycle_id: self.scope.test_cycle_id.clone(),
            test_execution_id: self.scope.test_execution_id.clone(),
            test_case_id: self.scope.test_case_id.clone(),
            template_id: self.scope.template_id.clone(),
        }
    }

    pub(crate) fn matches_scope(&self, scope: Option<&RecordScope>) -> bool {
        self.scope
            .test_cycle_id
            .as_ref()
            .is_none_or(|id| scope.and_then(|scope| scope.test_cycle_id.as_ref()) == Some(id))
            && self.scope.test_execution_id.as_ref().is_none_or(|id| {
                scope.and_then(|scope| scope.test_execution_id.as_ref()) == Some(id)
            })
            && self
                .scope
                .test_case_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.test_case_id.as_ref()) == Some(id))
            && self
                .scope
                .template_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.template_id.as_ref()) == Some(id))
            && self
                .scope
                .session_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.session_id.as_ref()) == Some(id))
            && self
                .scope
                .step_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.step_id.as_ref()) == Some(id))
            && self
                .scope
                .criterion_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.criterion_id.as_ref()) == Some(id))
            && self
                .scope
                .suite_id
                .as_ref()
                .is_none_or(|id| scope.and_then(|scope| scope.suite_id.as_ref()) == Some(id))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

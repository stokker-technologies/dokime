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

// Third Party Imports
use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};

// Local Imports
use crate::artifacts::ArtifactRecord;
use crate::document::definition::DocumentDefinition;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;

use dokime_domain::model::records::evidence::RecordEntry;

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

#[derive(Clone, Copy, Serialize)]
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
    Native(crate::records::view::View<'a>),
    Pages(&'a [harmos::storage::ReadBatch<Vec<RecordEntry>>]),
    Subscribed { rows: crate::records::view::View<'a>, captures: &'a dokime_domain::state::records::set::TestRecordSet },
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
                records: Records::Native((&self.data.records).into()),
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
            Self::Pages(pages) => pages.iter().map(|page| page.len()).sum(),
            Self::Subscribed { rows, .. } => rows.len(),
        }
    }
}

impl<'a> Records<'a> {
    pub(super) fn iter(self) -> impl DoubleEndedIterator<Item = RecordView<'a>> + Clone {
        let (native, pages, scopes) = match self {
            Self::Native(rows) => (rows, &[][..], None),
            Self::Pages(pages) => (crate::records::view::View::default(), pages, None),
            Self::Subscribed { rows, captures } => (rows, &[][..], Some(captures)),
        };
        native.iter().chain(pages.iter().flat_map(|page| page.iter())).map(move |entry| RecordView {
            entry,
            scope: scopes.and_then(|captures| captures.captured_scope(entry)).or(entry.scope.as_ref()),
        })
    }
}

impl Default for Records<'_> {
    fn default() -> Self {
        Self::Native(crate::records::view::View::default())
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
        native.iter().chain(outputs.iter().filter_map(|output| match &output.content {
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

impl DocumentSelection {
    pub(crate) fn matches_sample(&self, sample: &crate::telemetry::TelemetrySample) -> bool {
        self.matches_scope(sample.scope.as_ref()) || self.captured_sample_scope(sample).is_some()
    }

    pub(crate) fn captured_sample_scope<'a>(&self, sample: &'a crate::telemetry::TelemetrySample) -> Option<&'a RecordScope> {
        if self.scope.test_execution_id.is_none() && self.scope.test_cycle_id.is_none() {
            return None;
        }
        sample.captured_scopes.iter().find(|scope| self.matches_scope(Some(scope)))
    }

    pub(crate) fn foreign_execution_scope(&self, scope: Option<&RecordScope>) -> bool {
        self.scope.test_execution_id.is_some()
            && scope.is_some_and(|scope| {
                scope.test_execution_id == self.scope.test_execution_id
                    && (scope.test_cycle_id.as_ref().is_some_and(|id| self.scope.test_cycle_id.as_ref() != Some(id))
                        || scope.test_case_id.as_ref().is_some_and(|id| self.scope.test_case_id.as_ref() != Some(id)))
            })
    }

    pub(crate) fn evidence_scope(&self) -> crate::evidence_runtime::ScopeSelection {
        crate::evidence_runtime::ScopeSelection {
            test_cycle_id: self.scope.test_cycle_id.clone(),
            test_execution_id: self.scope.test_execution_id.clone(),
            test_case_id: self.scope.test_case_id.clone(),
            template_id: self.scope.template_id.clone(),
        }
    }

    pub(crate) fn matches_scope(&self, scope: Option<&RecordScope>) -> bool {
        self.scope.test_cycle_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.test_cycle_id.as_ref()) == Some(id))
            && self.scope.test_execution_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.test_execution_id.as_ref()) == Some(id))
            && self.scope.test_case_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.test_case_id.as_ref()) == Some(id))
            && self.scope.template_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.template_id.as_ref()) == Some(id))
            && self.scope.session_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.session_id.as_ref()) == Some(id))
            && self.scope.step_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.step_id.as_ref()) == Some(id))
            && self.scope.criterion_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.criterion_id.as_ref()) == Some(id))
            && self.scope.suite_id.as_ref().is_none_or(|id| scope.and_then(|scope| scope.suite_id.as_ref()) == Some(id))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

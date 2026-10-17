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
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::Write,
    num::NonZeroUsize,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

// Third Party Imports

// Local Imports
use super::{
    CapturedExecutionReportInput, CycleReportInput,
    cycle::Index,
    render::{cycle::Report, header::Header},
};
use crate::document::capture::Capture;
use crate::execution::context::captured::source::CapturedSource;
use crate::{
    runtime::Error, runtime::Result, installed_plugins::RuntimeAppearanceFrame, runtime::progress::Operation,
    runtime::storage::cycle::CycleSnapshot,
};
use crate::core::DocumentId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A report's frozen inputs, rendered to HTML on demand.
pub(super) struct HtmlReport {
    index: Index,
    source: Source,
    /// The measured length avoids counting the same immutable report twice.
    /// Zero until counted; a report is never empty.
    counted: AtomicUsize,
}

/* === Enums === */
enum Source {
    Saved {
        snapshot: CycleSnapshot,
        documents: BTreeMap<DocumentId, Capture>,
        members: Vec<Arc<CapturedExecutionReportInput>>,
    },
    Captured {
        input: Arc<CapturedExecutionReportInput>,
        evidence: Arc<CapturedSource>,
    },
}

pub(super) enum Member {
    Captured(Arc<CapturedExecutionReportInput>),
}

/* === Traits === */

/* === Implementations === */
impl Deref for Member {
    type Target = CycleReportInput;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Captured(input) => &input.input,
        }
    }
}

impl HtmlReport {
    pub(super) fn from_cycle(
        snapshot: CycleSnapshot,
        captured_members: Vec<CapturedExecutionReportInput>,
        operation: &Operation,
    ) -> Result<Self> {
        let count = captured_members.len();
        let documents = cycle_document(&snapshot)?;
        let mut members = Vec::with_capacity(count);
        let mut index = Index::new();
        // Compose Frozen Member Results Without Reading Pruned Raw Evidence.
        for (materialized, mut captured) in captured_members.into_iter().enumerate() {
            if captured.input.cycle.test_cycle_id != snapshot.source().test_cycle_id {
                return Err(invalid("materialized member belongs to another cycle"));
            }
            operation.progress(materialized as u64, count as u64);
            let own = captured.input.cycle.test_executions[0].test_case_id.clone();
            let id = captured.execution_id.clone();
            captured.record_contracts.clear();
            captured.telemetry_contracts.clear();
            let cycle = &mut captured.context.cycle;
            cycle.cases.retain(|case| case.test_case_id == own);
            cycle.cases.shrink_to_fit();
            cycle.execution_order.retain(|candidate| *candidate == id);
            cycle.execution_order.shrink_to_fit();
            index.push_capture(&captured)?;
            members.push(Arc::new(captured));
        }
        operation.progress(count as u64, count as u64);
        index.sort();
        Ok(Self {
            index,
            source: Source::Saved {
                snapshot,
                documents,
                members,
            },
            counted: AtomicUsize::new(0),
        })
    }

    pub(super) fn from_capture(
        captured: Arc<CapturedExecutionReportInput>,
        evidence: Arc<CapturedSource>,
    ) -> Result<Self> {
        let mut index = Index::new();
        index.push_capture(&captured)?;
        index.sort();
        Ok(Self {
            index,
            source: Source::Captured {
                input: captured,
                evidence,
            },
            counted: AtomicUsize::new(0),
        })
    }

    fn read(&self, member: usize) -> Result<Member> {
        match &self.source {
            Source::Captured { input, .. } if member == 0 => Ok(Member::Captured(input.clone())),
            Source::Captured { .. } => Err(invalid("captured report has only one member")),
            Source::Saved { members, .. } => members
                .get(member)
                .cloned()
                .map(Member::Captured)
                .ok_or_else(|| invalid("cycle report member is missing")),
        }
    }

    pub(super) fn header(&self) -> Header<'_> {
        match &self.source {
            Source::Saved {
                snapshot,
                documents,
                ..
            } => Header::cycle(snapshot, documents),
            Source::Captured { input, .. } => (&input.input).into(),
        }
    }

    fn appearance(&self) -> &RuntimeAppearanceFrame {
        match &self.source {
            Source::Saved { snapshot, .. } => snapshot.appearance(),
            Source::Captured { evidence, .. } => &evidence.context.appearance,
        }
    }

    pub(super) fn captured_input(&self) -> Result<&Arc<CapturedExecutionReportInput>> {
        match &self.source {
            Source::Captured { input, .. } => Ok(input),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution input")),
        }
    }

    pub(super) fn captured_source(&self) -> Result<&Arc<CapturedSource>> {
        match &self.source {
            Source::Captured { evidence, .. } => Ok(evidence),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution source")),
        }
    }

    pub(super) fn length(&self, maximum: NonZeroUsize) -> Result<usize> {
        let counted = self.counted.load(Ordering::Relaxed);
        if counted != 0 && counted <= maximum.get() {
            return Ok(counted);
        }
        let mut count = super::output::Count::new(maximum);
        match self.write(&mut count) {
            Ok(()) => {
                self.counted.store(count.len(), Ordering::Relaxed);
                Ok(count.len())
            }
            Err(_) if count.exceeded() => Err(Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "report output size",
            })),
            Err(error) => Err(error),
        }
    }

    pub(super) fn bytes(&self, length: usize) -> Result<Vec<u8>> {
        let mut buffer =
            super::output::Buffer::new(length).map_err(|source| invalid(&source.to_string()))?;
        self.write(&mut buffer)?;
        let bytes = buffer.into_bytes();
        if bytes.len() != length {
            return Err(invalid("report output changed its counted size"));
        }
        Ok(bytes)
    }

    pub(super) fn write(&self, writer: &mut dyn Write) -> Result<()> {
        let failure = RefCell::new(None);
        let read = |member| match self.read(member) {
            Ok(value) => Ok(value),
            Err(source) => {
                let message = source.to_string();
                *failure.borrow_mut() = Some(source);
                Err(Error::InvalidReportInputs(message))
            }
        };
        let report = Report {
            header: self.header(),
            index: &self.index,
            appearance: self.appearance(),
            read: &read,
        };
        let outcome = report.write(writer);
        if let Some(source) = failure.into_inner() {
            return Err(source);
        }
        outcome
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn cycle_document(snapshot: &CycleSnapshot) -> Result<BTreeMap<DocumentId, Capture>> {
    let source = snapshot.source();
    let mut scope = crate::core::RecordScope::default().with_cycle(source.test_cycle_id.clone());
    scope.session_id = Some(snapshot.session_id().clone());
    let declaration = crate::document::DocumentResolveInput {
        definition: crate::domains::documents::cycle_document(source.metadata.label.clone()),
        selection: crate::document::DocumentSelection { scope },
        data: Default::default(),
    };
    let capture = crate::document::bounded::capture_document_borrowed(
        declaration.borrowed(),
        Default::default(),
    )?;
    Ok(BTreeMap::from([(
        capture.definition.document_id.clone(),
        capture,
    )]))
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

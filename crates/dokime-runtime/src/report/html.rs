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
use std::{cell::RefCell, collections::BTreeMap, io::Write, ops::Deref, sync::Arc};

// Third Party Imports

// Local Imports
use super::{
    CapturedExecutionReportInput, CycleReportInput,
    cycle::{CycleMember, Index},
    render::{cycle::Report, header::Header},
};
use crate::document::capture::Capture;
use crate::execution::context::captured::source::CapturedSource;
use crate::{
    installed_plugins::RuntimeAppearanceFrame, runtime::Error, runtime::Result, runtime::progress::Operation,
    runtime::storage::cycle::CycleSnapshot,
};
use dokime_domain::model::identity::DocumentId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct HtmlReport {
    index: Index,
    source: Source,
}

/* === Enums === */

enum Source {
    Saved { snapshot: CycleSnapshot, documents: BTreeMap<DocumentId, Capture>, members: Vec<Arc<CapturedExecutionReportInput>> },
    Captured { input: Arc<CapturedExecutionReportInput>, evidence: Arc<CapturedSource> },
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
    pub(super) fn from_cycle(snapshot: CycleSnapshot, cycle_members: Vec<CycleMember>, operation: &Operation) -> Result<Self> {
        let count = cycle_members.len();
        let documents = cycle_document(&snapshot)?;
        let mut members = Vec::with_capacity(count);
        let mut index = Index::new();
        // Compose Frozen Member Results Without Reading Pruned Raw Evidence.
        for (materialized, member) in cycle_members.into_iter().enumerate() {
            operation.progress(materialized as u64, count as u64);
            let mut captured = match member {
                CycleMember::Captured(captured) => captured,
                CycleMember::Planned(execution) => {
                    if execution.test_cycle_id != snapshot.source().test_cycle_id {
                        return Err(invalid("planned member belongs to another cycle"));
                    }
                    index.push_planned(&execution, snapshot.captured_at())?;
                    continue;
                }
            };
            if captured.input.cycle.test_cycle_id != snapshot.source().test_cycle_id {
                return Err(invalid("materialized member belongs to another cycle"));
            }
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
            members.push(Arc::new(*captured));
        }
        operation.progress(count as u64, count as u64);
        index.sort();
        Ok(Self { index, source: Source::Saved { snapshot, documents, members } })
    }

    pub(super) fn from_capture(captured: Arc<CapturedExecutionReportInput>, evidence: Arc<CapturedSource>) -> Result<Self> {
        let mut index = Index::new();
        index.push_capture(&captured)?;
        index.sort();
        Ok(Self { index, source: Source::Captured { input: captured, evidence } })
    }

    fn read(&self, member: usize) -> Result<Member> {
        match &self.source {
            Source::Captured { input, .. } if member == 0 => Ok(Member::Captured(input.clone())),
            Source::Captured { .. } => Err(invalid("captured report has only one member")),
            Source::Saved { members, .. } => {
                members.get(member).cloned().map(Member::Captured).ok_or_else(|| invalid("cycle report member is missing"))
            }
        }
    }

    pub(super) fn header(&self) -> Header<'_> {
        match &self.source {
            Source::Saved { snapshot, documents, .. } => Header::cycle(snapshot, documents),
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

    // One Serialization Pass Yields The Published Bytes; Their Length And Digest Are Taken From Them.
    pub(super) fn render(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(super::asset::REPORT_APP_TEMPLATE.len());
        self.write(&mut bytes)?;
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
        let report = Report { header: self.header(), index: &self.index, appearance: self.appearance(), read: &read };
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
    let mut scope = dokime_domain::model::execution::RecordScope::default().with_cycle(source.test_cycle_id.clone());
    scope.session_id = Some(snapshot.session_id().clone());
    let declaration = crate::document::DocumentResolveInput {
        definition: crate::domains::documents::cycle_document(source.metadata.label.clone()),
        selection: crate::document::DocumentSelection { scope },
        data: Default::default(),
    };
    let capture = crate::document::bounded::capture_document_borrowed(declaration.borrowed(), Default::default())?;
    Ok(BTreeMap::from([(capture.definition.document_id.clone(), capture)]))
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

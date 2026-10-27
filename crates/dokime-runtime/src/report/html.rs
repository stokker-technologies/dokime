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
use std::{cell::RefCell, collections::BTreeMap, io::Write, ops::Deref};

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
    index: harmos::storage::ReadBatch<Index>,
    source: Source,
}

/* === Enums === */

enum Source {
    Saved {
        report_defaults: harmos::storage::ReadBatch<crate::config::ReportDefaults>,
        snapshot: CycleSnapshot,
        documents: harmos::storage::ReadBatch<BTreeMap<DocumentId, Capture>>,
        members: harmos::storage::ReadBatch<Vec<harmos::storage::ReadBatch<CapturedExecutionReportInput>>>,
    },
    Captured {
        input: harmos::storage::ReadBatch<CapturedExecutionReportInput>,
        evidence: harmos::storage::ReadBatch<CapturedSource>,
    },
}

pub(super) enum Member {
    Captured(harmos::storage::ReadBatch<CapturedExecutionReportInput>),
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
        snapshot: CycleSnapshot, cycle_members: Vec<CycleMember>, operation: &Operation, report_defaults: &crate::config::ReportDefaults,
    ) -> Result<Self> {
        let report_defaults = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(*report_defaults))?;
        let count = cycle_members.len();
        let documents = cycle_document(&snapshot, crate::document::bounded::DocumentLimits::default())?;
        let mut members = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::with_capacity(count)))?;
        let mut index = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Index::new()))?;
        // Compose Member Results Materialized From Their Saved Context And Evidence.
        for (materialized, member) in cycle_members.into_iter().enumerate() {
            operation.progress(materialized as u64, count as u64);
            let captured = match member {
                CycleMember::Captured(captured) => captured,
                CycleMember::Planned(execution) => {
                    if execution.test_cycle_id != snapshot.source().test_cycle_id {
                        return Err(invalid("planned member belongs to another cycle"));
                    }
                    index = index.try_map(|mut index| {
                        index.push_planned(&execution, snapshot.captured_at())?;
                        Ok::<_, Error>(index)
                    })?;
                    continue;
                }
            };
            if captured.input.cycle.test_cycle_id != snapshot.source().test_cycle_id {
                return Err(invalid("materialized member belongs to another cycle"));
            }
            index = index.try_map(|mut index| {
                index.push_capture(&captured)?;
                Ok::<_, Error>(index)
            })?;
            members = members.try_map(|mut members| {
                members.push(captured);
                Ok::<_, Error>(members)
            })?;
        }
        operation.progress(count as u64, count as u64);
        let index = index.try_map(|mut index| {
            index.sort();
            Ok::<_, Error>(index)
        })?;
        Ok(Self { index, source: Source::Saved { report_defaults, snapshot, documents, members } })
    }

    pub(super) fn from_capture(
        captured: harmos::storage::ReadBatch<CapturedExecutionReportInput>, evidence: harmos::storage::ReadBatch<CapturedSource>,
    ) -> Result<Self> {
        let index = harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut index = Index::new();
            index.push_capture(&captured)?;
            index.sort();
            Ok::<_, Error>(index)
        })?;
        Ok(Self { index, source: Source::Captured { input: captured, evidence } })
    }

    fn report_defaults(&self) -> &crate::config::ReportDefaults {
        match &self.source {
            Source::Saved { report_defaults, .. } => report_defaults,
            Source::Captured { input, .. } => &input.report_defaults,
        }
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

    pub(super) fn captured_input(&self) -> Result<&harmos::storage::ReadBatch<CapturedExecutionReportInput>> {
        match &self.source {
            Source::Captured { input, .. } => Ok(input),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution input")),
        }
    }

    pub(super) fn captured_source(&self) -> Result<&harmos::storage::ReadBatch<CapturedSource>> {
        match &self.source {
            Source::Captured { evidence, .. } => Ok(evidence),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution source")),
        }
    }

    // One Serialization Pass Yields The Published Bytes; Their Length And Digest Are Taken From Them.
    pub(super) fn render(&self) -> Result<harmos::storage::ReadBatch<Vec<u8>>> {
        harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut bytes = Vec::new();
            self.write(&mut bytes)?;
            Ok::<_, Error>(bytes)
        })
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
            report_defaults: self.report_defaults(),
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

fn cycle_document(
    snapshot: &CycleSnapshot, limits: crate::document::bounded::DocumentLimits,
) -> Result<harmos::storage::ReadBatch<BTreeMap<DocumentId, Capture>>> {
    let source = snapshot.source();
    let identity = snapshot.session_id();
    let declaration = harmos::storage::ReadBatch::new(()).try_map(|_| {
        let cycle = source.test_cycle_id.clone();
        let mut scope = dokime_domain::model::execution::RecordScope::default().with_cycle(cycle);
        scope.session_id = Some(identity.clone());
        let label = source.metadata.label.clone();
        let definition = crate::domains::documents::cycle_document(label);
        Ok::<_, Error>(crate::document::DocumentResolveInput {
            definition,
            selection: crate::document::DocumentSelection { scope },
            data: Default::default(),
        })
    })?;

    declaration.try_map(|declaration| {
        let capture = crate::document::bounded::capture_document_borrowed(declaration.borrowed(), limits)?;
        let mut documents = BTreeMap::new();
        documents.insert(capture.definition.document_id.clone(), capture);
        Ok::<_, Error>(documents)
    })
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

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
use std::{cell::RefCell, collections::BTreeMap, io::Write, num::NonZeroUsize, ops::Deref};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{SharedQueryValue, StorageProductionPreparation, StorageView},
};

// Local Imports
use super::{
    CapturedExecutionReportInput, CycleReportInput,
    cycle::{Index, allowance},
    render::{cycle::Report, header::Header},
};
use crate::document::capture::Capture;
use crate::execution::context::source::CapturedSource;
use crate::{
    Error, Result, RuntimeHandle,
    installed_plugins::RuntimeAppearanceFrame,
    runtime::storage::{cycle::CycleSnapshot, execution::ExecutionView},
};
use dokime::{core::DocumentId, storage::Manager};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct HtmlReport {
    pub(super) manager: Manager,
    index: SharedQueryValue<Index>,
    source: Source,
    _memory: StorageProductionPreparation,
}

struct Members {
    values: Vec<SharedQueryValue<CapturedExecutionReportInput>>,
}

impl MemorySize for Members {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.values.capacity().saturating_mul(
            std::mem::size_of::<SharedQueryValue<CapturedExecutionReportInput>>(),
        ))
    }
}

/* === Enums === */
enum Source {
    Saved {
        snapshot: CycleSnapshot,
        documents: SharedQueryValue<BTreeMap<DocumentId, Capture>>,
        executor: tokio::runtime::Handle,
        members: SharedQueryValue<Members>,
    },
    Captured {
        input: SharedQueryValue<CapturedExecutionReportInput>,
        evidence: SharedQueryValue<CapturedSource>,
    },
}

pub(super) enum Member {
    Captured(SharedQueryValue<CapturedExecutionReportInput>),
}

pub(super) enum Computation {
    Saved(harmos::storage::StorageViewComputation),
    Projection,
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
impl Computation {
    pub(super) fn check(&self) -> harmos::Result<()> {
        match self {
            Self::Saved(work) => work.check(),
            Self::Projection => Ok(()),
        }
    }
}

impl HtmlReport {
    pub(super) async fn from_cycle(
        runtime: &RuntimeHandle,
        snapshot: CycleSnapshot,
    ) -> Result<Self> {
        let manager = runtime.storage();
        let revisions = snapshot.revisions();
        let documents = cycle_document(&snapshot)?;
        let memory = manager.prepare_production(allowance(std::mem::size_of::<Self>())?)?;
        let executor = tokio::runtime::Handle::try_current()
            .map_err(|error| Error::Operation(error.to_string()))?;
        let bytes = std::mem::size_of::<Members>().saturating_add(revisions.len().saturating_mul(
            std::mem::size_of::<SharedQueryValue<CapturedExecutionReportInput>>(),
        ));
        let mut members = manager
            .produce(allowance(bytes)?, |_| {
                Ok(Members {
                    values: Vec::with_capacity(revisions.len()),
                })
            })?
            .try_into_value()?;
        let index = Index::new();
        let mut index = manager
            .produce(allowance(index.memory_size())?, |_| Ok(index))?
            .try_into_value()?;
        // Each member revision is read under its own default-limit view; read
        // them all before any materialized member is retained beside one.
        let mut opened = Vec::new();
        opened
            .try_reserve_exact(revisions.len())
            .map_err(|_| invalid("cycle report member memory overflows"))?;
        for member in 0..revisions.len() {
            opened.push(snapshot.member(manager, member).await?);
        }
        for execution in opened {
            // Members Repeat The Frozen Catalogs And Every Case Of The Cycle; Once
            // Materialized, The Cycle Report Keeps Only Its Own Case So Retained
            // Members Grow Linearly Instead Of With Catalog And Cycle Size.
            let captured = super::materialize::execution(runtime, &execution, true).await?;
            let own = captured.input.cycle.test_executions[0].test_case_id.clone();
            let id = captured.execution.execution_id.clone();
            let bytes = allowance(captured.memory_size())?;
            let captured = captured
                .try_map(bytes, |mut captured, _| {
                    captured.record_contracts.clear();
                    captured.telemetry_contracts.clear();
                    let cycle = &mut captured.context.cycle;
                    cycle.cases.retain(|case| case.test_case_id == own);
                    cycle.cases.shrink_to_fit();
                    cycle.execution_order.retain(|candidate| *candidate == id);
                    cycle.execution_order.shrink_to_fit();
                    Ok(captured)
                })?
                .try_into_shared()?;
            let growth = allowance(index.capture_growth(&captured.input))?;
            index = index.try_map(growth, |mut index, _| {
                index.push_capture(&captured.input).map_err(failure)?;
                Ok(index)
            })?;
            let bytes = allowance(members.memory_size())?;
            members = members.try_map(bytes, |mut members, _| {
                members.values.push(captured);
                Ok(members)
            })?;
        }
        let bytes = allowance(index.memory_size())?;
        let index = index
            .try_map(bytes, |mut index, _| {
                index.sort();
                Ok(index)
            })?
            .try_into_shared()?;
        Ok(Self {
            manager: manager.clone(),
            index,
            source: Source::Saved {
                snapshot,
                documents,
                executor,
                members: members.try_into_shared()?,
            },
            _memory: memory,
        })
    }
}
impl HtmlReport {
    fn read(&self, member: usize) -> Result<Member> {
        match &self.source {
            Source::Captured { input, .. } if member == 0 => Ok(Member::Captured(input.clone())),
            Source::Captured { .. } => Err(invalid("captured report has only one member")),
            Source::Saved { members, .. } => members
                .values
                .get(member)
                .cloned()
                .map(Member::Captured)
                .ok_or_else(|| invalid("cycle report member is missing")),
        }
    }

    fn computation(&self) -> Result<Computation> {
        match &self.source {
            Source::Saved { snapshot, .. } => {
                Ok(Computation::Saved(snapshot.view().begin_computation()?))
            }
            Source::Captured { .. } => Ok(Computation::Projection),
        }
    }

    pub(super) fn saved_source(&self) -> Result<(&StorageView, &tokio::runtime::Handle)> {
        match &self.source {
            Source::Saved {
                snapshot, executor, ..
            } => Ok((snapshot.view(), executor)),
            Source::Captured { .. } => Err(invalid("captured report has no saved view")),
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

    pub(super) async fn from_snapshot(
        runtime: &RuntimeHandle,
        snapshot: &ExecutionView,
        final_report: bool,
    ) -> Result<Self> {
        let snapshot = snapshot.snapshot(runtime.storage()).await?;
        let materialized = super::materialize::execution(runtime, &snapshot, final_report)
            .await?
            .try_into_shared()?;
        let evidence = snapshot
            .captured_source()
            .ok_or_else(|| invalid("execution snapshot context is missing"))?;
        Self::from_capture(runtime.storage(), &materialized, evidence)
    }

    pub(super) fn captured_input(&self) -> Result<&SharedQueryValue<CapturedExecutionReportInput>> {
        match &self.source {
            Source::Captured { input, .. } => Ok(input),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution input")),
        }
    }

    pub(super) fn captured_source(&self) -> Result<&SharedQueryValue<CapturedSource>> {
        match &self.source {
            Source::Captured { evidence, .. } => Ok(evidence),
            Source::Saved { .. } => Err(invalid("cycle report has no single execution source")),
        }
    }

    pub(super) fn from_capture(
        manager: &Manager,
        captured: &SharedQueryValue<CapturedExecutionReportInput>,
        evidence: &SharedQueryValue<CapturedSource>,
    ) -> Result<Self> {
        captured.check_origin(manager)?;
        evidence.check_origin(manager)?;
        let index = Index::new();
        let bytes = index.capture_growth(&captured.input);
        let index = captured
            .try_project(allowance(bytes)?, |input, _| {
                let mut index = index;
                index.push_capture(&input.input).map_err(failure)?;
                index.sort();
                Ok(index)
            })?
            .try_into_shared()?;
        let memory = manager.prepare_production(allowance(std::mem::size_of::<Self>())?)?;
        Ok(Self {
            manager: manager.clone(),
            index,
            source: Source::Captured {
                input: captured.clone(),
                evidence: evidence.clone(),
            },
            _memory: memory,
        })
    }

    pub(super) fn length(&self, maximum: NonZeroUsize) -> Result<usize> {
        let mut count = super::output::Count::new(maximum);
        self.write(&mut count)?;
        Ok(count.len())
    }

    pub(super) fn bytes(&self, length: usize) -> Result<Vec<u8>> {
        let mut buffer =
            super::output::Buffer::new(length).map_err(|source| invalid(&source.to_string()))?;
        // The Caller Projects These Bytes From The Same Original Capture Loan.
        self.write_inner(&mut buffer)?;
        let bytes = buffer.into_bytes();
        if bytes.len() != length {
            return Err(invalid("report output changed its counted size"));
        }
        Ok(bytes)
    }

    pub(super) fn write(&self, writer: &mut dyn Write) -> Result<()> {
        if let Source::Captured { input, .. } = &self.source {
            input.try_project(NonZeroUsize::new(1).unwrap(), |_, _| {
                self.write_inner(writer).map_err(failure)
            })?;
            return Ok(());
        }
        self.write_inner(writer)
    }

    fn write_inner(&self, writer: &mut dyn Write) -> Result<()> {
        let _scratch = self
            .manager
            .prepare_production(allowance(super::render::header_memory(self.header()))?)?;
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
            manager: &self.manager,
            computation: &|| self.computation(),
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
    snapshot: &CycleSnapshot,
) -> Result<SharedQueryValue<BTreeMap<DocumentId, Capture>>> {
    let source = snapshot.source();
    let session = snapshot.session_id();
    let definition_bytes = crate::capabilities::documents::cycle_memory(&source.metadata.label)
        .saturating_add(std::mem::size_of::<crate::document::DocumentResolveInput>())
        .saturating_add(source.test_cycle_id.memory_size())
        .saturating_add(session.memory_size());
    let declaration =
        snapshot
            .view()
            .project_shared(source, allowance(definition_bytes)?, |cycle, _| {
                let mut scope =
                    dokime::core::RecordScope::default().with_cycle(cycle.test_cycle_id.clone());
                scope.session_id = Some(session.clone());
                Ok(crate::document::DocumentResolveInput {
                    definition: crate::capabilities::documents::cycle_document(
                        cycle.metadata.label.clone(),
                    ),
                    selection: crate::document::DocumentSelection { scope },
                    data: Default::default(),
                })
            })?;
    let bytes =
        crate::document::bounded::capture_memory(&declaration.borrowed(), Default::default())?;
    let capture = declaration.try_map(allowance(bytes)?, |input, _| {
        crate::document::bounded::capture_document_borrowed(input.borrowed(), Default::default())
            .map_err(|error| failure(error.into()))
    })?;
    let bytes = harmos::btree_map_memory::<DocumentId, Capture>(1)
        .saturating_add(capture.memory_size())
        .saturating_add(capture.definition.document_id.memory_size());
    Ok(capture
        .try_map(allowance(bytes)?, |capture, _| {
            Ok(BTreeMap::from([(
                capture.definition.document_id.clone(),
                capture,
            )]))
        })?
        .try_into_shared()?)
}

fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

fn failure(source: Error) -> harmos::Error {
    harmos::Error::backend(
        "assemble saved HTML report",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */

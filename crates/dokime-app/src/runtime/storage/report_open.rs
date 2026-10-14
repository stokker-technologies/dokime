/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Open Report: the one report a cycle or an execution has right now.
//
// A sealed subject answers with the bytes its finalization published. A
// subject still in live state, running or left unsealed by a runtime that
// stopped before finalization, answers with a report rendered from its
// current evidence. A sealed subject whose published file can no longer be
// read answers with a report rendered again from its sealed evidence. The
// last two are never written back: the seal stays the stable boundary.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        FileReference, ObjectVersion, QueryValue, SharedQueryValue, StorageFileReader, StorageView,
    },
};
// Local Imports
use super::{cycle::CycleSnapshot, cycle_finalization, execution::ExecutionView};
use crate::{Error, Result, RuntimeHandle, execution::CycleView};
use dokime::{
    core::{Metadata, TestCycleId, TestExecutionId, Timestamp},
    execution::{ExecutionStatus, TestCycle},
};

/* === Constants === */
/// The largest report Open Report delivers, sealed or rendered.
pub const MAX_OPENED_REPORT_BYTES: usize = 16 * 1024 * 1024;
const READ_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// One report, ready to deliver.
pub struct OpenedReport {
    pub test_cycle_id: TestCycleId,
    /// Set when the report is one execution's rather than its cycle's.
    pub test_execution_id: Option<TestExecutionId>,
    /// The cycle's metadata, for a cycle report.
    pub metadata: Option<Metadata>,
    pub generated_at: Timestamp,
    pub origin: ReportOrigin,
    pub html: QueryValue<Vec<u8>>,
}

/* === Enums === */
/// Where an opened report came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportOrigin {
    /// The report finalization sealed, read back unchanged.
    Sealed,
    /// The sealed report could not be read, so it was rendered again from the
    /// sealed evidence.
    Regenerated,
    /// The subject is not sealed: rendered from its current evidence.
    Current,
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// The report a cycle has right now. See the module comment for which.
    pub async fn open_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let live = self
            .watch_state()
            .latest()
            .state()
            .cycles
            .get(id)
            .map(|cycle| cycle.saved.is_none());
        match live {
            Some(true) => self.current_cycle_report(id).await,
            _ => self.sealed_cycle_report(id).await,
        }
    }

    /// The report one execution has right now. See the module comment for
    /// which.
    pub async fn open_execution_report(&self, id: &TestExecutionId) -> Result<OpenedReport> {
        let live = self
            .watch_state()
            .latest()
            .state()
            .executions
            .get(id)
            .map(|execution| (execution.saved.is_none(), execution.test_cycle_id.clone()));
        if let Some((true, cycle)) = live {
            let view = self.execution_snapshot(id).await?;
            let html = crate::report::rendered::execution(self, &view, false, maximum()).await?;
            return Ok(OpenedReport {
                test_cycle_id: cycle,
                test_execution_id: Some(id.clone()),
                metadata: None,
                generated_at: Timestamp::now_utc(),
                origin: ReportOrigin::Current,
                html,
            });
        }
        let reads = self.config().storage_reads.execution;
        let view = self.storage().view_with_limits(reads).await?;
        let limit = view.decoded_bytes()?;
        let inspection =
            ExecutionView::open(self.storage(), view, id, ObjectVersion::Latest, limit)
                .await?
                .filter(|inspection| inspection.execution().saved.is_some())
                .ok_or_else(|| not_found("execution_report_missing", "execution has no report"))?;
        let execution = inspection.execution();
        let seal = execution.saved.as_ref().expect("filtered sealed execution");
        let (generated_at, test_cycle_id) =
            (seal.sealed_at.clone(), execution.test_cycle_id.clone());
        let read = match inspection.read_file(&seal.report).await {
            Ok(reader) => read_whole(self, reader, &seal.report).await,
            Err(error) => Err(error),
        };
        let (origin, html) = match read {
            Ok(html) => (ReportOrigin::Sealed, html),
            Err(error) => {
                tracing::warn!(%error, execution_id = %id, "sealed execution report is unreadable; rendering it again");
                let html =
                    crate::report::rendered::execution(self, &inspection, true, maximum()).await?;
                (ReportOrigin::Regenerated, html)
            }
        };
        Ok(OpenedReport {
            test_cycle_id,
            test_execution_id: Some(id.clone()),
            metadata: None,
            generated_at,
            origin,
            html,
        })
    }

    async fn current_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let runtime = self.clone();
        let cycle = id.clone();
        // Hold The Save Order So Finalization Cannot Seal A Member Mid-Capture.
        let snapshot = tokio::spawn(async move {
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let source = runtime.current_cycle_source(&cycle)?;
            if source.saved.is_some() {
                return Err(invalid("cycle sealed while its report was being captured"));
            }
            let mut members = Vec::new();
            members
                .try_reserve_exact(source.test_executions.len())
                .map_err(|_| exhausted())?;
            // An Attempt That Never Started Has No Evidence To Report Yet.
            for execution in source
                .test_executions
                .iter()
                .filter(|execution| execution.status != ExecutionStatus::Pending)
            {
                members.push(runtime.snapshot_member(&source, execution).await?);
            }
            let view = runtime
                .storage()
                .view_with_limits(runtime.config().storage_reads.execution)
                .await?;
            let header = runtime.current_header(&source)?;
            let appearance = runtime.member_appearance(&members, &view).await?;
            let node = runtime.state_read().identity.node.node_id.clone();
            CycleSnapshot::current(runtime.storage(), view, header, members, appearance, &node)
        })
        .await??;
        let metadata = snapshot.source().metadata.clone();
        let html = crate::report::rendered::cycle(self, snapshot, maximum()).await?;
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(metadata),
            generated_at: Timestamp::now_utc(),
            origin: ReportOrigin::Current,
            html,
        })
    }

    async fn sealed_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let storage = self.storage();
        let mut query =
            harmos::storage::ObjectQuery::<dokime::storage::objects::cycle::Cycle>::new();
        query.item = Some(id.as_str());
        let selection = storage.query(query).await?;
        let view = selection
            .view()
            .ok_or_else(|| invalid("cycle query has no saved view"))?
            .clone();
        let selected = dokime::storage::objects::cycle::read_in(
            storage,
            &view,
            id,
            ObjectVersion::Latest,
            selection.decoded_bytes(),
        )
        .await?
        .ok_or_else(|| not_found("cycle_not_found", "cycle not found"))?;
        let cycle = &selected.value;
        let seal = cycle
            .saved
            .as_ref()
            .ok_or_else(|| not_found("cycle_report_missing", "cycle has no sealed report"))?;
        seal.validate(cycle).map_err(dokime::storage::Error::from)?;
        let read = match view.read_file(&seal.report).await {
            Ok(reader) => read_whole(self, reader, &seal.report).await,
            Err(error) => Err(error.into()),
        };
        let (origin, html) = match read {
            Ok(html) => (ReportOrigin::Sealed, html),
            Err(error) => {
                tracing::warn!(%error, cycle_id = %id, "sealed cycle report is unreadable; rendering it again");
                let snapshot = self.sealed_snapshot(cycle).await?;
                let html = crate::report::rendered::cycle(self, snapshot, maximum()).await?;
                (ReportOrigin::Regenerated, html)
            }
        };
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(cycle.metadata.clone()),
            generated_at: seal.sealed_at.clone(),
            origin,
            html,
        })
    }

    /// The snapshot the seal was cut from: the same header and the same exact
    /// member revisions, so the rendering reads the sealed evidence only.
    async fn sealed_snapshot(&self, cycle: &TestCycle) -> Result<CycleSnapshot> {
        let seal = cycle.saved.as_ref().expect("sealed cycle");
        let storage = self.storage();
        let view = storage
            .view_with_limits(self.config().storage_reads.execution)
            .await?;
        let limit = view.decoded_bytes()?;
        let source = storage
            .produce(bound(cycle.memory_size())?, |_| {
                let mut source = cycle.clone();
                source.saved = None;
                Ok(source)
            })?
            .try_into_value()?
            .try_into_shared()?;
        let revisions = storage
            .produce(bound(seal.executions.memory_size())?, |_| {
                Ok(seal.executions.clone())
            })?
            .try_into_value()?
            .try_into_shared()?;
        let mut members = Vec::new();
        if let Some(first) = seal.executions.first() {
            let member = ExecutionView::open(
                storage,
                view.clone(),
                &first.execution_id,
                ObjectVersion::Exact(first.version.clone()),
                limit,
            )
            .await?
            .ok_or_else(|| invalid("sealed cycle member is missing"))?;
            members.push(member.snapshot(storage).await?);
        }
        let appearance = self.member_appearance(&members, &view).await?;
        CycleSnapshot::capture(
            storage,
            view,
            source,
            revisions,
            appearance,
            &seal.owner_node_id,
        )
    }

    fn current_cycle_source(&self, id: &TestCycleId) -> Result<CycleView> {
        let state = self.state_read();
        state
            .cycle_view(id)
            .ok_or_else(|| not_found("cycle_not_found", "cycle not found"))
    }

    fn current_header(&self, source: &CycleView) -> Result<SharedQueryValue<TestCycle>> {
        let session = match &source.session_id {
            Some(session) => session.clone(),
            None => self.watch_state().latest().state().session_id.clone(),
        };
        Ok(self
            .storage()
            .produce(
                bound(
                    cycle_finalization::header_memory(source).saturating_add(session.memory_size()),
                )?,
                |_| {
                    let mut header = cycle_finalization::header(source, None);
                    header.session_id = Some(session.clone());
                    Ok(header)
                },
            )?
            .try_into_value()?
            .try_into_shared()?)
    }

    /// The appearance the first member captured, as finalization takes it, or
    /// the runtime's own when no member has run.
    async fn member_appearance(
        &self,
        members: &[ExecutionView],
        view: &StorageView,
    ) -> Result<SharedQueryValue<crate::installed_plugins::RuntimeAppearanceFrame>> {
        if let Some(member) = members.first() {
            let appearance = &member.context()?.appearance;
            return Ok(self
                .storage()
                .produce(bound(appearance.memory_size())?, |_| Ok(appearance.clone()))?
                .try_into_value()?
                .try_into_shared()?);
        }
        Ok(self
            .report_appearance(view, view.decoded_bytes()?)
            .await?
            .try_into_shared()?)
    }
}

/* === Public Functions === */

/* === Private Functions === */
/// Reads a sealed report whole, so a missing or damaged file is known before
/// a single byte is delivered and the caller can render it again instead.
async fn read_whole(
    runtime: &RuntimeHandle,
    mut reader: StorageFileReader,
    file: &FileReference,
) -> Result<QueryValue<Vec<u8>>> {
    let length = usize::try_from(file.length)
        .ok()
        .filter(|length| *length <= MAX_OPENED_REPORT_BYTES)
        .ok_or_else(|| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "sealed report size",
            })
        })?;
    let mut html = runtime
        .storage()
        .produce(
            bound(length.saturating_add(std::mem::size_of::<Vec<u8>>()))?,
            |_| {
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(length).map_err(|_| exhausted())?;
                Ok(bytes)
            },
        )?
        .try_into_value()?;
    let chunk = NonZeroUsize::new(READ_CHUNK_BYTES).expect("nonzero report chunk");
    let capacity = bound(length.saturating_add(std::mem::size_of::<Vec<u8>>()))?;
    while let Some(read) = reader.next_chunk(chunk).await? {
        html = html.try_map(capacity, |mut bytes, _| {
            if bytes.len().saturating_add(read.as_bytes().len()) > length {
                return Err(exhausted());
            }
            bytes.extend_from_slice(read.as_bytes());
            Ok(bytes)
        })?;
    }
    if html.len() != length {
        return Err(invalid("sealed report ended before its length"));
    }
    Ok(html)
}

fn maximum() -> NonZeroUsize {
    NonZeroUsize::new(MAX_OPENED_REPORT_BYTES).expect("nonzero report limit")
}

fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| invalid("opened report memory overflows"))
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "opened report memory",
    }
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

fn not_found(code: &str, message: &str) -> Error {
    Error::NotFound {
        code: code.into(),
        message: message.into(),
    }
}

/* === Tests === */

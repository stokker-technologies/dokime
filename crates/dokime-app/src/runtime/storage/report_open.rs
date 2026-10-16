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
// stopped before finalization, materializes at its pending frozen boundary,
// or the current persisted cursor when no finalization is pending.
// An unreadable sealed report returns an error: finalized
// evidence may already have been pruned, and its published bytes are authoritative.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::{FileReference, ObjectVersion, QueryValue, StorageFileReader};
// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result, RuntimeHandle, runtime::progress::Operation};
use dokime::core::{Metadata, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */
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
    /// The subject is not sealed: materialized at its selected persisted boundary.
    Current,
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// The report a cycle has right now. See the module comment for which.
    pub async fn open_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let operation = self.operation_progress().begin("Opening Cycle Report");
        let opened = self.open_cycle_report_once(id, &operation).await;
        operation.track(opened)
    }

    /// The report one execution has right now. See the module comment for
    /// which.
    pub async fn open_execution_report(&self, id: &TestExecutionId) -> Result<OpenedReport> {
        let operation = self.operation_progress().begin("Opening Execution Report");
        let opened = self.open_execution_report_once(id, &operation).await;
        operation.track(opened)
    }

    async fn open_cycle_report_once(
        &self,
        id: &TestCycleId,
        operation: &Operation,
    ) -> Result<OpenedReport> {
        let live = self
            .watch_state()
            .latest()
            .state()
            .cycles
            .get(id)
            .map(|cycle| cycle.saved.is_none());
        match live {
            Some(true) => self.current_cycle_report(id, operation).await,
            _ => self.sealed_cycle_report(id, operation).await,
        }
    }

    async fn open_execution_report_once(
        &self,
        id: &TestExecutionId,
        operation: &Operation,
    ) -> Result<OpenedReport> {
        let live = self
            .watch_state()
            .latest()
            .state()
            .executions
            .get(id)
            .map(|execution| (execution.saved.is_none(), execution.test_cycle_id.clone()));
        if let Some((true, cycle)) = live {
            operation.phase("Capturing Evidence");
            let at = self
                .state_read()
                .report_finalizations
                .get(id)
                .map(|intent| intent.evidence_at.clone())
                .unwrap_or_else(|| self.storage().cursor_saved());
            let materialized = self
                .operations()
                .call_as::<dokime::operations::reports::MaterializeReport>(
                    dokime::operations::reports::MaterializeReportRequest {
                        subject: dokime::operations::reports::ReportSubject::Execution(
                            id.to_string(),
                        ),
                        at,
                    },
                )
                .await?;
            let reader = self.storage().read_file(&materialized.report).await?;
            let html = read_whole(self, reader, &materialized.report).await?;
            return Ok(OpenedReport {
                test_cycle_id: cycle,
                test_execution_id: Some(id.clone()),
                metadata: None,
                generated_at: Timestamp::now_utc(),
                origin: ReportOrigin::Current,
                html,
            });
        }
        operation.phase("Reading Sealed Report");
        let reads = self.config().storage.interactive;
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
        let reader = inspection.read_file(&seal.report).await?;
        let html = read_whole(self, reader, &seal.report).await?;
        Ok(OpenedReport {
            test_cycle_id,
            test_execution_id: Some(id.clone()),
            metadata: None,
            generated_at,
            origin: ReportOrigin::Sealed,
            html,
        })
    }

    async fn current_cycle_report(
        &self,
        id: &TestCycleId,
        operation: &Operation,
    ) -> Result<OpenedReport> {
        operation.phase("Materializing Report");
        let metadata = self
            .state_read()
            .cycle_view(id)
            .ok_or_else(|| not_found("cycle_not_found", "cycle not found"))?
            .metadata
            .clone();
        let at = self
            .execution_storage
            .cycle_materialization_cursor(id)
            .await?
            .unwrap_or_else(|| self.storage().cursor_saved());
        let materialized = self
            .operations()
            .call_as::<dokime::operations::reports::MaterializeReport>(
                dokime::operations::reports::MaterializeReportRequest {
                    subject: dokime::operations::reports::ReportSubject::Cycle(id.to_string()),
                    at,
                },
            )
            .await?;
        let reader = self.storage().read_file(&materialized.report).await?;
        let html = read_whole(self, reader, &materialized.report).await?;
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(metadata),
            generated_at: Timestamp::now_utc(),
            origin: ReportOrigin::Current,
            html,
        })
    }

    async fn sealed_cycle_report(
        &self,
        id: &TestCycleId,
        operation: &Operation,
    ) -> Result<OpenedReport> {
        operation.phase("Reading Sealed Report");
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
        let reader = view.read_file(&seal.report).await?;
        let html = read_whole(self, reader, &seal.report).await?;
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(cycle.metadata.clone()),
            generated_at: seal.sealed_at.clone(),
            origin: ReportOrigin::Sealed,
            html,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
/// Reads a sealed report whole so missing or truncated content fails before
/// any bytes are delivered.
async fn read_whole(
    runtime: &RuntimeHandle,
    mut reader: StorageFileReader,
    file: &FileReference,
) -> Result<QueryValue<Vec<u8>>> {
    let length = usize::try_from(file.length).ok().ok_or_else(|| {
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
    let batch_bytes = runtime.config().storage.response.batch_bytes;
    let capacity = bound(length.saturating_add(std::mem::size_of::<Vec<u8>>()))?;
    while let Some(read) = reader.next_chunk(batch_bytes).await? {
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

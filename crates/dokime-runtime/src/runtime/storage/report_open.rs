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
// Third Party Imports
use harmos::storage::ObjectVersion;
// Local Imports
use super::execution::ExecutionView;
use crate::{runtime::Error, runtime::Result, RuntimeHandle};
use crate::core::{Metadata, TestCycleId, TestExecutionId, Timestamp};

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
    pub html: Vec<u8>,
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
    /// A live report is one materialization operation; a sealed one is read.
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
            let at = self
                .state_read()
                .report_finalizations
                .get(id)
                .map(|intent| intent.evidence_at.clone())
                .unwrap_or_else(|| self.storage().cursor_saved());
            let materialized = self
                .operations()
                .call_as::<crate::runtime::operations::reports::MaterializeReport>(
                    crate::runtime::operations::reports::MaterializeReportRequest {
                        subject: crate::runtime::operations::reports::ReportSubject::Execution(
                            id.to_string(),
                        ),
                        at,
                    },
                )
                .await?;
            let html =
                crate::storage::read_file_bytes(self.storage(), &materialized.report).await?;
            return Ok(OpenedReport {
                test_cycle_id: cycle,
                test_execution_id: Some(id.clone()),
                metadata: None,
                generated_at: Timestamp::now_utc(),
                origin: ReportOrigin::Current,
                html,
            });
        }
        let inspection = ExecutionView::open(self.storage(), id, ObjectVersion::Latest)
            .await?
            .filter(|inspection| inspection.execution().saved.is_some())
            .ok_or_else(|| not_found("execution_report_missing", "execution has no report"))?;
        let execution = inspection.execution();
        let seal = execution.saved.as_ref().expect("filtered sealed execution");
        let (generated_at, test_cycle_id) =
            (seal.sealed_at.clone(), execution.test_cycle_id.clone());
        let html = crate::storage::read_file_bytes(self.storage(), &seal.report).await?;
        Ok(OpenedReport {
            test_cycle_id,
            test_execution_id: Some(id.clone()),
            metadata: None,
            generated_at,
            origin: ReportOrigin::Sealed,
            html,
        })
    }

    async fn current_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
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
            .call_as::<crate::runtime::operations::reports::MaterializeReport>(
                crate::runtime::operations::reports::MaterializeReportRequest {
                    subject: crate::runtime::operations::reports::ReportSubject::Cycle(id.to_string()),
                    at,
                },
            )
            .await?;
        let html = crate::storage::read_file_bytes(self.storage(), &materialized.report).await?;
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
        let selected = crate::storage::objects::cycle::read(storage, id, ObjectVersion::Latest)
            .await?
            .ok_or_else(|| not_found("cycle_not_found", "cycle not found"))?;
        let cycle = &selected.value;
        let seal = cycle
            .saved
            .as_ref()
            .ok_or_else(|| not_found("cycle_report_missing", "cycle has no sealed report"))?;
        seal.validate(cycle).map_err(crate::storage::Error::from)?;
        let html = crate::storage::read_file_bytes(storage, &seal.report).await?;
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
fn not_found(code: &str, message: &str) -> Error {
    Error::NotFound {
        code: code.into(),
        message: message.into(),
    }
}

/* === Tests === */

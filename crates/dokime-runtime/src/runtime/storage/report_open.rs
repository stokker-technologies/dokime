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
// A sealed subject answers with the bytes its finalization published, also
// when it is an execution sealed inside a cycle that is still running. A
// subject still in live state, running or left unsealed by a runtime that
// stopped before finalization, materializes at its pending frozen boundary,
// or the current persisted cursor when no finalization is pending. A download
// never refuses because the final report already exists.
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
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct OpenedReport {
    pub test_cycle_id: TestCycleId,
    pub test_execution_id: Option<TestExecutionId>,
    pub metadata: Option<Metadata>,
    pub generated_at: Timestamp,
    pub origin: ReportOrigin,
    pub file: harmos::storage::FileReference,
    pub reader: harmos::storage::FileReader,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportOrigin {
    Sealed,
    Current,
}

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn open_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let live = self.watch_state().latest().state().cycles.get(id).map(|cycle| cycle.saved.is_none());
        match live {
            Some(true) => self.current_cycle_report(id).await,
            _ => self.sealed_cycle_report(id).await,
        }
    }

    pub async fn open_execution_report(&self, id: &TestExecutionId) -> Result<OpenedReport> {
        let live = self
            .watch_state()
            .latest()
            .state()
            .executions
            .get(id)
            .map(|execution| (execution.saved.is_none(), execution.test_cycle_id.clone()));
        if let Some((true, cycle)) = live {
            // An Execution Sealed Inside A Running Cycle Serves Its Final Report Directly.
            if let Some(seal) = self.execution_storage.durable_seal(id).await? {
                let session = self.storage().clone();
                let file = seal.report.clone();
                let reader = report_reader(&session, &file).await?;
                return Ok(OpenedReport {
                    test_cycle_id: cycle,
                    test_execution_id: Some(id.clone()),
                    metadata: None,
                    generated_at: seal.sealed_at.clone(),
                    origin: ReportOrigin::Sealed,
                    file,
                    reader,
                });
            }
            let at = self
                .state_read()
                .report_finalizations
                .get(id)
                .map(|intent| intent.evidence_at.clone())
                .unwrap_or_else(|| self.storage().cursor_saved());
            let materialized = self
                .operations()
                .call::<crate::operations::reports::MaterializeReport>(crate::operations::reports::MaterializeReport {
                    subject: dokime_domain::model::reports::wire::ReportSubject::Execution(id.to_string()),
                    at,
                })
                .await?;
            let session = self.storage().clone();
            let file = materialized.report.clone();
            let reader = report_reader(&session, &file).await?;
            return Ok(OpenedReport {
                test_cycle_id: cycle,
                test_execution_id: Some(id.clone()),
                metadata: None,
                generated_at: crate::model::time::now_utc(),
                origin: ReportOrigin::Current,
                file,
                reader,
            });
        }
        let session = self.storage().clone();
        let inspection = ExecutionView::open(self.storage(), id, ObjectVersion::Latest)
            .await?
            .filter(|inspection| inspection.execution().saved.is_some())
            .ok_or_else(|| not_found("execution_report_missing", "execution has no report"))?;
        let execution = inspection.execution();
        let seal = execution.saved.as_ref().expect("filtered sealed execution");
        let (generated_at, test_cycle_id) = (seal.sealed_at.clone(), execution.test_cycle_id.clone());
        let file = seal.report.clone();
        let reader = report_reader(&session, &file).await?;
        Ok(OpenedReport {
            test_cycle_id,
            test_execution_id: Some(id.clone()),
            metadata: None,
            generated_at,
            origin: ReportOrigin::Sealed,
            file,
            reader,
        })
    }

    async fn current_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let metadata = self.state_read().cycle_view(id).ok_or_else(|| not_found("cycle_not_found", "cycle not found"))?.metadata.clone();
        let at = self.execution_storage.cycle_materialization_cursor(id).await?.unwrap_or_else(|| self.storage().cursor_saved());
        let materialized = self
            .operations()
            .call::<crate::operations::reports::MaterializeReport>(crate::operations::reports::MaterializeReport {
                subject: dokime_domain::model::reports::wire::ReportSubject::Cycle(id.to_string()),
                at,
            })
            .await?;
        let session = self.storage().clone();
        let file = materialized.report.clone();
        let reader = report_reader(&session, &file).await?;
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(metadata),
            generated_at: crate::model::time::now_utc(),
            origin: ReportOrigin::Current,
            file,
            reader,
        })
    }

    async fn sealed_cycle_report(&self, id: &TestCycleId) -> Result<OpenedReport> {
        let storage = self.storage();
        let session = storage.clone();
        let selected = crate::storage::objects::cycle::read(&session, id, ObjectVersion::Latest)
            .await?
            .ok_or_else(|| not_found("cycle_not_found", "cycle not found"))?;
        let cycle = &selected.value;
        let seal = cycle.saved.as_ref().ok_or_else(|| not_found("cycle_report_missing", "cycle has no sealed report"))?;
        seal.validate(cycle).map_err(crate::storage::Error::from)?;
        let file = seal.report.clone();
        let reader = report_reader(&session, &file).await?;
        Ok(OpenedReport {
            test_cycle_id: id.clone(),
            test_execution_id: None,
            metadata: Some(cycle.metadata.clone()),
            generated_at: seal.sealed_at.clone(),
            origin: ReportOrigin::Sealed,
            file,
            reader,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn report_reader(storage: &crate::storage::Manager, file: &harmos::storage::FileReference) -> Result<harmos::storage::FileReader> {
    Ok(storage
        .read_file(harmos::storage::FileRead {
            source: harmos::storage::FileSource::Stored(file.clone()),
            range: harmos::storage::ByteRange { offset: 0, limit: None },
        })
        .await?)
}

fn not_found(code: &str, message: &str) -> Error {
    Error::Admission(AdmissionFailure::NotFound {
        code: code.into(),
        message: message.into(),
    })
}

/* === Tests === */

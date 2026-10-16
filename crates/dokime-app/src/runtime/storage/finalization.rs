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
use super::execution::ExecutionView;
use crate::{
    Error, Result, RuntimeHandle,
    execution::{CycleView, TestExecution},
};
use dokime::core::{TestExecutionId, Timestamp};
use dokime::execution::ExecutionStatus;
use dokime::operations::reports::{MaterializeReport, MaterializeReportRequest, ReportSubject};
use futures_util::{StreamExt, stream::FuturesUnordered};
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn finish_saved_executions(
        &self,
        cycle: &CycleView,
        executions: Vec<TestExecution>,
    ) -> Result<Vec<(TestExecutionId, Error)>> {
        let mut remaining = executions.iter();
        let mut active = FuturesUnordered::new();
        for execution in remaining.by_ref().take(4) {
            active.push(self.finish_saved_execution_attributed(cycle, execution));
        }
        let mut failures = Vec::new();
        while let Some((execution, result)) = active.next().await {
            if let Err(error) = result {
                failures.push((execution.test_execution_id.clone(), error));
            }
            if let Some(execution) = remaining.next() {
                active.push(self.finish_saved_execution_attributed(cycle, execution));
            }
        }
        Ok(failures)
    }

    pub(crate) async fn unsealed_members(
        &self,
        cycle: &CycleView,
        selected: &[TestExecution],
    ) -> Vec<TestExecution> {
        let candidates = cycle
            .test_executions
            .iter()
            .filter(|execution| {
                execution.saved.is_none()
                    && execution.finished_at.is_some()
                    && matches!(
                        execution.status,
                        ExecutionStatus::Passed
                            | ExecutionStatus::Failed
                            | ExecutionStatus::Blocked
                            | ExecutionStatus::Cancelled
                    )
                    && !selected
                        .iter()
                        .any(|chosen| chosen.test_execution_id == execution.test_execution_id)
            })
            .collect::<Vec<_>>();
        let mut unsealed = Vec::new();
        for execution in candidates {
            let id = &execution.test_execution_id;
            if self.execution_storage.has_final_report(id).await {
                continue;
            }
            match self.saved_seal(id).await {
                Ok(false) => unsealed.push(execution.clone()),
                Ok(true) => {}
                Err(error) => {
                    tracing::debug!(%error, execution_id = %id, "earlier attempt seal is unreadable");
                }
            }
        }
        unsealed
    }

    async fn saved_seal(&self, id: &TestExecutionId) -> Result<bool> {
        let latest =
            ExecutionView::open(self.storage(), id, harmos::storage::ObjectVersion::Latest).await?;
        Ok(latest.is_some_and(|latest| latest.execution().saved.is_some()))
    }

    async fn finish_saved_execution_attributed<'a>(
        &self,
        cycle: &CycleView,
        execution: &'a TestExecution,
    ) -> (&'a TestExecution, Result<()>) {
        (
            execution,
            self.finish_saved_execution(cycle, execution).await,
        )
    }

    pub(crate) async fn finish_saved_execution(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<()> {
        let id = &execution.test_execution_id;
        if execution.saved.is_some() {
            return Err(invalid("sealed execution must read its published report"));
        }
        // A Previous Attempt May Have Persisted Its Seal Before Managed Adoption Failed.
        if self.saved_seal(id).await? {
            return Ok(());
        }
        let intent = self
            .state_manager
            .state()
            .read(|state| state.report_finalizations.get(id).cloned())
            .await?
            .value;
        let at = if let Some(intent) = intent {
            intent.evidence_at
        } else {
            // Normal Completion May Await Accepted Samples; Stop Already Has Its Earlier Intent.
            let evidence_at = self.storage().flush().await?;
            self.commit(crate::transactions::RequestReportFinalization {
                executions: vec![id.clone()],
                intent: crate::transactions::ReportFinalization {
                    evidence_at: evidence_at.clone(),
                    requested_at: Timestamp::now_utc(),
                    lifecycle: None,
                },
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
            evidence_at
        };
        if self.execution_storage.acquisition(id).await.is_none() {
            self.execution_storage.acquire(cycle, execution).await?;
        }
        let gate = self.execution_storage.materialization_gate(id).await?;
        self.capture_report_lifecycle(execution).await?;
        self.operations
            .call_as::<MaterializeReport>(MaterializeReportRequest {
                subject: ReportSubject::Execution(id.to_string()),
                at: at.clone(),
            })
            .await?;
        let _guard = gate.lock().await;
        // Only Finalization Seals The Subject, After The Full Publication Is Durable.
        self.execution_storage.seal_final_report(id).await?;
        Ok(())
    }
}
/* === Public Functions === */
/* === Private Functions === */
fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}
/* === Tests === */

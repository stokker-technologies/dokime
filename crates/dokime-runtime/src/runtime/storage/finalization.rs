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

// Third Party Imports
use futures_util::{StreamExt, stream::FuturesUnordered};

// Local Imports
use crate::RuntimeHandle;
use crate::operations::reports::MaterializeReport;
use crate::runtime::Error;
use crate::runtime::Result;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::reports::wire::ReportSubject;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub(crate) async fn seal_settled_cycle(&self, id: &dokime_domain::model::identity::TestCycleId) -> Result<()> {
        let cycle = {
            let state = self.state_read();
            let cycle = state.cycle_view(id).ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
            if state.run_requests.values().any(|request| request.summary.cycle_id == *id && !request.summary.phase.terminal()) {
                return Err(invalid("a running request still owns this cycle"));
            }
            cycle
        };
        if cycle.saved.is_some() {
            return Ok(());
        }
        for execution in &cycle.test_executions {
            if execution.saved.is_none() {
                self.finish_saved_execution(&cycle, execution).await?;
            }
        }
        let progress = self.operation_progress().begin("Sealing Cycle Report");
        let sealed = match self.finish_saved_cycle(&cycle, &progress).await {
            Ok(sealed) => {
                progress.finish();
                sealed
            }
            Err(error) => {
                progress.fail(&error);
                return Err(error);
            }
        };
        self.commit(crate::transactions::SealCycle { sealed_cycle: sealed.header.clone() })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        self.execution_storage.retire_cycle_report(id).await;
        Ok(())
    }

    pub(crate) async fn finish_saved_executions(
        &self, cycle: &CycleView, executions: Vec<TestExecution>,
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

    async fn finish_saved_execution_attributed<'a>(
        &self, cycle: &CycleView, execution: &'a TestExecution,
    ) -> (&'a TestExecution, Result<()>) {
        (execution, self.finish_saved_execution(cycle, execution).await)
    }

    pub(crate) async fn finish_saved_execution(&self, cycle: &CycleView, execution: &TestExecution) -> Result<()> {
        let id = &execution.test_execution_id;
        if execution.saved.is_some() {
            return Ok(());
        }
        // A Previous Attempt May Have Persisted Its Seal Before Managed Adoption Failed.
        if self.adopt_saved_execution(id).await? {
            return Ok(());
        }
        let intent = self.state_manager.state().read(|state| state.report_finalizations.get(id).cloned()).await?.value;
        let at = if let Some(intent) = intent {
            intent.evidence_at
        } else {
            // Normal Completion May Await Accepted Samples; Stop Already Has Its Earlier Intent.
            let evidence_at = self.storage().flush().await?;
            let requested = self
                .commit(crate::transactions::RequestReportFinalization {
                    executions: vec![id.clone()],
                    intent: crate::transactions::ReportFinalization {
                        evidence_at: evidence_at.clone(),
                        requested_at: crate::model::time::now_utc(),
                        lifecycle: None,
                    },
                })
                .await;
            if let Err(error) = requested {
                if self.adopt_saved_execution(id).await? {
                    return Ok(());
                }
                return Err(Error::Operation(error.to_string()));
            }
            let captured = self
                .state_manager
                .state()
                .read(|state| state.report_finalizations.get(id).map(|intent| intent.evidence_at.clone()))
                .await?
                .value;
            let Some(captured) = captured else {
                if self.adopt_saved_execution(id).await? {
                    return Ok(());
                }
                return Err(invalid("execution finalization intent disappeared before sealing"));
            };
            captured
        };
        if self.execution_storage.acquisition(id).await.is_none() {
            self.execution_storage.acquire(cycle, execution).await?;
        }
        let gate = self.execution_storage.materialization_gate(id).await?;
        self.capture_report_lifecycle(execution).await?;
        self.operations.call(MaterializeReport { subject: ReportSubject::Execution(id.to_string()), at: at.clone() }).await?;
        let _guard = gate.lock().await;
        // Only Finalization Seals The Subject, After The Full Publication Is Durable.
        self.execution_storage.seal_final_report(id).await?;
        let seal = self.execution_storage.durable_seal(id).await?.ok_or_else(|| invalid("execution seal is not durable"))?;
        self.commit(crate::transactions::SealExecution { seal }).await.map_err(|error| Error::Operation(error.to_string()))?;
        Ok(())
    }

    async fn adopt_saved_execution(&self, id: &TestExecutionId) -> Result<bool> {
        let Some(seal) = self.execution_storage.durable_seal(id).await? else {
            return Ok(false);
        };
        self.commit(crate::transactions::SealExecution { seal }).await.map_err(|error| Error::Operation(error.to_string()))?;
        Ok(true)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

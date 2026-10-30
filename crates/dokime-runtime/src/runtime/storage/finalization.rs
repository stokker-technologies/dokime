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
use futures_util::{FutureExt, StreamExt, stream::FuturesUnordered};

// Local Imports
use crate::RuntimeHandle;
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

impl super::executions::ExecutionStorage {
    // Execution Reports Need The Cycle Header And Their Own Member Only.
    pub(crate) async fn report_member_source(&self, id: &TestExecutionId) -> Result<harmos::storage::ReadBatch<CycleView>> {
        let (mut source, lifecycle) = self
            .state_manager
            .state()
            .read(|state| {
                let member = state.executions.get(id).ok_or_else(|| invalid("report execution is not active"))?;
                let cycle = state.cycles.get(&member.test_cycle_id).ok_or_else(|| invalid("report cycle is not active"))?;
                let source = CycleView {
                    saved: cycle.saved.clone(),
                    test_cycle_id: cycle.test_cycle_id.clone(),
                    session_id: cycle.session_id.clone(),
                    metadata: cycle.metadata.clone(),
                    phase: cycle.phase,
                    lifecycle: cycle.lifecycle,
                    created_at: cycle.created_at.clone(),
                    started_at: cycle.started_at.clone(),
                    updated_at: cycle.updated_at.clone(),
                    test_executions: vec![member.clone()],
                    records: cycle.records.clone(),
                    continue_automatically: cycle.continue_automatically,
                    result: cycle.result.clone(),
                };
                let lifecycle = state.report_finalizations.get(id).and_then(|intent| intent.lifecycle.clone());
                Ok::<_, Error>((source, lifecycle))
            })
            .await?
            .value?;

        if let Some(lifecycle) = lifecycle {
            match self.frozen_execution(&self.storage, &lifecycle.execution).await? {
                super::executions::FrozenExecution::Live(member) => source.test_executions[0] = (*member).clone(),
                super::executions::FrozenExecution::Saved(member) => {
                    return member.try_map(|member| {
                        source.test_executions[0] = member.value;
                        Ok(source)
                    });
                }
            }
        }
        Ok(harmos::storage::ReadBatch::new(source))
    }
}

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
        self.finish_saved_members(&cycle).await?;
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
        self.commit(crate::transactions::SealCycle { sealed_cycle: (*sealed.header).clone() })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        self.execution_storage.retire_cycle_report(id).await;
        Ok(())
    }

    // Unsealed Members Use The Configured Report Member Concurrency. Those Without An Evidence
    // Boundary Share One Flush And One Durable Intent: Their Providers Have
    // Settled, So No Later Sample Can Belong To Any Of Them.
    pub(crate) async fn finish_saved_members(&self, cycle: &CycleView) -> Result<()> {
        let pending = cycle.test_executions.iter().filter(|execution| execution.saved.is_none()).collect::<Vec<_>>();
        if pending.is_empty() {
            return Ok(());
        }
        let unbounded = {
            let state = self.state_read();
            pending
                .iter()
                .map(|execution| &execution.test_execution_id)
                .filter(|id| {
                    !state.report_finalizations.contains_key(*id)
                        && state.executions.get(*id).is_some_and(|execution| execution.saved.is_none())
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        if !unbounded.is_empty() {
            let evidence_at = self.storage().flush().await?;
            let requested = self
                .commit(crate::transactions::RequestReportFinalization {
                    executions: unbounded,
                    intent: crate::transactions::ReportFinalization {
                        failure: None,
                        evidence_at,
                        requested_at: crate::model::time::now_utc(),
                        lifecycle: None,
                    },
                })
                .await;
            // A Refused Batch Leaves Each Member To Request Its Own Boundary.
            if let Err(error) = requested {
                tracing::debug!(%error, cycle_id = %cycle.test_cycle_id, "batched report finalization intent was refused");
            }
        }
        match self.finish_saved_executions(cycle, &pending).await?.into_iter().next() {
            Some((_, error)) => Err(error),
            None => Ok(()),
        }
    }

    pub(crate) async fn finish_saved_executions(
        &self, cycle: &CycleView, executions: &[&TestExecution],
    ) -> Result<Vec<(TestExecutionId, Error)>> {
        let mut remaining = executions.iter().copied();
        let mut active = FuturesUnordered::new();
        let active_members = self.execution_storage.report_concurrency.get();
        for execution in remaining.by_ref().take(active_members) {
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

    pub(crate) async fn queue_saved_execution(&self, cycle: &CycleView, execution: &TestExecution) -> Result<()> {
        let id = &execution.test_execution_id;
        if execution.saved.is_some() {
            return Ok(());
        }
        // Serialize Scheduling Only; Rendering Never Holds This Registry Lock.
        let mut jobs = self.execution_storage.report_jobs.lock().await;
        if jobs.contains_key(id) {
            return Ok(());
        }
        if self.execution_storage.acquisition(id).await.is_none() {
            self.execution_storage.acquire(cycle, execution).await?;
        }
        self.execution_storage.finish_outputs(id).await?;
        let evidence_at = self.storage().flush().await?;
        if !self.state_read().report_finalizations.contains_key(id) {
            self.commit(crate::transactions::RequestReportFinalization {
                executions: vec![id.clone()],
                intent: crate::transactions::ReportFinalization {
                    failure: None,
                    evidence_at: evidence_at.clone(),
                    requested_at: crate::model::time::now_utc(),
                    lifecycle: None,
                },
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        }
        self.capture_report_lifecycle(execution).await?;
        let at = self.state_read().report_finalizations[id].evidence_at.clone();
        // Freeze And Drain Native Evidence Before Returning; HTML Is Independent.
        self.execution_storage.prepare_capture(cycle, execution, &at, true).await?;
        self.storage().flush_recovery().await?;
        let runtime = self.clone();
        let cycle = self.execution_storage.report_member_source(id).await?;
        let execution = execution.clone();
        let workers = self.execution_storage.report_workers.clone();
        let job = tokio::spawn(async move {
            let _worker = workers.acquire_owned().await.expect("report workers remain open");
            let rendered = std::panic::AssertUnwindSafe(runtime.materialize_saved_execution(&cycle, &execution)).catch_unwind().await;
            let rendered = rendered.unwrap_or_else(|_| Err(Error::Operation("report worker panicked".into())));
            if let Err(error) = rendered {
                tracing::error!(execution_id = %execution.test_execution_id, %error, "background report failed");
                if let Err(failure) = runtime
                    .commit(crate::transactions::FailReport {
                        execution: execution.test_execution_id.clone(),
                        cause: error.to_string(),
                    })
                    .await
                {
                    tracing::error!(%failure, "report failure state could not be saved");
                }
            }
        });
        jobs.insert(id.clone(), job);
        Ok(())
    }

    pub(crate) async fn stop_report_jobs(&self) {
        let jobs = std::mem::take(&mut *self.execution_storage.report_jobs.lock().await);
        for job in jobs.values() {
            job.abort();
        }
        for (_, job) in jobs {
            let _ = job.await;
        }
    }

    pub async fn wait_execution_report(&self, id: &TestExecutionId) -> Result<()> {
        let mut watch = self.watch_state();
        let mut shutdown = self.shutdown_requested.subscribe();
        loop {
            let snapshot = watch.latest();
            let execution = snapshot.state().executions.get(id).ok_or_else(|| invalid("report execution is missing"))?;
            if execution.saved.is_some() {
                return Ok(());
            }
            if let Some(cause) = snapshot.state().report_finalizations.get(id).and_then(|intent| intent.failure.as_ref()) {
                return Err(Error::InvalidReportInputs(cause.clone()));
            }
            drop(snapshot);
            if *shutdown.borrow() {
                return Err(Error::ShuttingDown);
            }
            tokio::select! {
                changed = watch.next() => { changed.ok_or_else(|| invalid("runtime stopped before report publication"))?; }
                _ = shutdown.changed() => return Err(Error::ShuttingDown),
            }
        }
    }

    async fn finish_saved_execution_attributed<'a>(
        &self, cycle: &CycleView, execution: &'a TestExecution,
    ) -> (&'a TestExecution, Result<()>) {
        (execution, self.finish_saved_execution(cycle, execution).await)
    }

    #[tracing::instrument(name = "execution.finalize", skip_all)]
    pub(crate) async fn finish_saved_execution(&self, cycle: &CycleView, execution: &TestExecution) -> Result<()> {
        let queued = self.execution_storage.report_jobs.lock().await.contains_key(&execution.test_execution_id);
        if queued {
            return self.wait_execution_report(&execution.test_execution_id).await;
        }
        self.materialize_saved_execution(cycle, execution).await
    }

    async fn materialize_saved_execution(&self, cycle: &CycleView, execution: &TestExecution) -> Result<()> {
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
                        failure: None,
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
        self.capture_report_lifecycle(execution).await?;
        crate::runtime::operations::reports::Resources::from_runtime(self)
            .materialize(crate::operations::reports::MaterializeReport {
                subject: ReportSubject::Execution(id.to_string()),
                at: at.clone(),
            })
            .await?;
        Ok(())
    }

    pub(crate) async fn complete_report_publication(&self, id: &TestExecutionId) -> Result<()> {
        self.execution_storage.seal_final_report(id).await?;
        let seal = self.execution_storage.durable_seal(id).await?.ok_or_else(|| invalid("execution seal is not durable"))?;
        self.commit_admitted_seal(seal).await?;
        Ok(())
    }

    async fn adopt_saved_execution(&self, id: &TestExecutionId) -> Result<bool> {
        let Some(seal) = self.execution_storage.durable_seal(id).await? else {
            return Ok(false);
        };
        self.commit_admitted_seal(seal).await?;
        Ok(true)
    }

    async fn commit_admitted_seal(
        &self, seal: harmos::storage::ReadBatch<dokime_domain::model::execution::saved::ExecutionSeal>,
    ) -> Result<()> {
        let transaction = seal.try_map(|seal| {
            Ok::<_, crate::storage::Error>(crate::transactions::Transactions::from(crate::transactions::SealExecution { seal }))
        })?;
        let owner = transaction.owner();
        self.commit_owned(harmos::operations::Response::from_owner(owner)).await.map_err(|error| Error::Operation(error.to_string()))?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

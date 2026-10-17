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
pub(crate) use crate::transactions::{ControlCycle as Control, CycleControlAction as Action};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::state::RequestKey;
// Local Imports
use crate::{CommandContext, RuntimeHandle, runtime::Error, unavailable_rejection};
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// Fence Measurements And Remember The Already-Persisted Boundary.
    pub(crate) async fn freeze_report_inputs(&self, ids: Vec<TestExecutionId>) -> crate::runtime::Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        self.sampling().fence_executions(&ids).await;
        let evidence_at = self.storage().cursor_saved();
        self.commit(crate::transactions::RequestReportFinalization {
            executions: ids,
            intent: crate::transactions::ReportFinalization { evidence_at, requested_at: crate::model::time::now_utc(), lifecycle: None },
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        Ok(())
    }

    /// Freeze the terminal `execution` a final report captures: saved once as
    /// its durable revision, which the lifecycle intent references.
    pub(crate) async fn capture_report_lifecycle(
        &self, execution: &dokime_domain::model::execution::result::TestExecution,
    ) -> crate::runtime::Result<crate::transactions::ReportLifecycle> {
        let captured = |runtime: &Self| {
            runtime.state_read().report_finalizations.get(&execution.test_execution_id).and_then(|intent| intent.lifecycle.clone())
        };
        if let Some(lifecycle) = captured(self) {
            self.execution_storage.adopt_revision(execution, &lifecycle.execution).await?;
            return Ok(lifecycle);
        }
        let revision = self.execution_storage.freeze_revision(execution).await?;
        self.commit(crate::transactions::CaptureReportLifecycle {
            lifecycle: crate::transactions::ReportLifecycle { id: uuid::Uuid::now_v7(), execution: (*revision).clone() },
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        captured(self).ok_or_else(|| crate::runtime::Error::Operation("report lifecycle capture was not applied".into()))
    }

    pub async fn pause_execution(&self, id: TestExecutionId, context: CommandContext) -> Result<(), Error> {
        self.control_execution(id, Action::Pause, context).await
    }

    pub async fn resume_execution(&self, id: TestExecutionId, context: CommandContext) -> Result<(), Error> {
        self.control_execution(id, Action::Resume, context).await
    }

    pub async fn stop_execution(&self, id: TestExecutionId, context: CommandContext) -> Result<(), Error> {
        self.control_execution(id, Action::Stop, context).await
    }

    async fn control_execution(&self, id: TestExecutionId, action: Action, context: CommandContext) -> Result<(), Error> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        let permit = self.admit_owned_work().ok_or(Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            runtime.control_execution_inner(id, action, context).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    async fn control_execution_inner(&self, id: TestExecutionId, action: Action, context: CommandContext) -> Result<(), Error> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        let _admission = admit_control(&action, &self.evidence_admission).await;
        let transaction =
            crate::transactions::ControlExecution { test_execution_id: id.clone(), action, occurred_at: crate::model::time::now_utc() };
        let (request_id, cycle_id) = {
            let state = self.state_read();
            let request = transaction.request(&state).map_err(|error| Error::conflict("execution_control_refused", error))?;
            (request.summary.request_id.clone(), request.summary.cycle_id.clone())
        };
        if matches!(action, Action::Stop) {
            self.freeze_report_inputs(vec![id.clone()]).await?;
        }
        self.commit_with_key(context.idempotency_key, transaction).await?;
        drop(_admission);

        let summary = self.state_read().run_requests[&request_id].summary.clone();
        self.request_storage
            .update(&request_id, |receipt| {
                if !receipt.summary.phase.terminal() {
                    receipt.summary.paused = summary.paused;
                    receipt.summary.stop_requested = summary.stop_requested;
                }
            })
            .await?;
        self.execution_storage.save_cycle_header(cycle_id.clone()).await?;
        self.reconcile_sampling().await;

        let cycle = self.state_read().cycle_view(&cycle_id).ok_or_else(|| Error::not_found("not_found", "execution cycle not found"))?;
        if cycle.test_executions.iter().any(|execution| {
            execution.test_execution_id == id && execution.status == dokime_domain::model::execution::result::ExecutionStatus::Running
        }) {
            self.execution_storage.checkpoint_targets(&cycle, &[id]).await?;
        }
        Ok(())
    }

    pub async fn pause_cycle(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Pause, context).await
    }

    pub async fn resume_cycle(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Resume, context).await
    }

    pub async fn stop_cycle(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Stop, context).await
    }

    async fn control_cycle(&self, test_cycle_id: TestCycleId, action: Action, context: CommandContext) -> Result<(), Error> {
        let runtime = self.clone();
        // The committed transition owns all of its checkpoint work, including
        // later executions if the caller stops waiting after the first one.
        tokio::spawn(async move { runtime.control_cycle_inner(test_cycle_id, action, context).await })
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?
    }

    async fn control_cycle_inner(&self, test_cycle_id: TestCycleId, action: Action, context: CommandContext) -> Result<(), Error> {
        // Stop must reach the provider even while an unrelated evidence write
        // or historical deletion holds admission on a slow storage device.
        // The journal still validates and serializes the control transition;
        // the terminal receipt still waits for the final evidence checkpoint.
        let _admission = admit_control(&action, &self.evidence_admission).await;
        // A Recovered Cycle Whose Report Intent Is Durable Only Awaits Its Seal.
        let pending_seal = {
            let state = self.state_read();
            state.cycle_report_finalizations.contains_key(&test_cycle_id)
                && state.cycles.get(&test_cycle_id).is_some_and(|cycle| cycle.saved.is_none())
        };
        if matches!(action, Action::Stop) && pending_seal {
            return self.seal_settled_cycle(&test_cycle_id).await;
        }
        let occurred_at = crate::model::time::now_utc();
        let transaction = Control { test_cycle_id: test_cycle_id.clone(), action, occurred_at: occurred_at.clone() };
        if matches!(action, Action::Stop) {
            harmos::state::Transaction::check(&transaction, &*self.state_read())
                .map_err(|reason| Error::conflict("cycle_control_refused", reason))?;
            let ids = self
                .state_read()
                .executions
                .values()
                .filter(|execution| {
                    execution.test_cycle_id == test_cycle_id && execution.saved.is_none() && execution.finished_at.is_none()
                })
                .map(|execution| execution.test_execution_id.clone())
                .collect();
            self.freeze_report_inputs(ids).await?;
        }
        let committed = self.commit_with_key(context.idempotency_key, transaction).await?;
        let bounded = self
            .state_read()
            .run_requests
            .values()
            .any(|request| request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal());
        if bounded && matches!(action, Action::Pause | Action::Resume) {
            let scopes = self
                .state_read()
                .executions
                .values()
                .filter(|execution| execution.test_cycle_id == test_cycle_id)
                .filter(|execution| execution.status == dokime_domain::model::execution::result::ExecutionStatus::Running)
                .map(|execution| execution.scope.clone())
                .collect::<Vec<_>>();
            for scope in scopes {
                let mut request = dokime_domain::model::records::publication::RecordPublishRequest::new(
                    if matches!(action, Action::Pause) {
                        crate::records::BuiltinRecordType::TestExecutionPaused
                    } else {
                        crate::records::BuiltinRecordType::TestExecutionResumed
                    }
                    .identifier(),
                    crate::records::RecordSeverity::Info,
                );
                let key = RequestKey::new(format!(
                    "cycle-control-record:{}:{}",
                    committed.position.0,
                    scope.test_execution_id.as_ref().expect("execution scope"),
                ));
                request.scope = Some(scope);
                self.commit_keyed(key, crate::transactions::PublishRecord::host(request, occurred_at.clone()))
                    .await
                    .map_err(|error| unavailable_rejection(error.to_string()))?;
            }
        }
        let request = self
            .state_read()
            .run_requests
            .values()
            .find(|request| request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal())
            .map(|request| request.summary.clone());
        if let Some(summary) = request {
            let id = summary.request_id.clone();
            self.request_storage
                .update(&id, |receipt| {
                    receipt.summary.paused = summary.paused;
                    receipt.summary.stop_requested = summary.stop_requested;
                })
                .await
                .map_err(|error| unavailable_rejection(error.to_string()))?;
        }
        // A Stop No Run Will Finish Seals The Cycle From Its Persisted Members,
        // Unless A Never-Started Draft Remains: Drafts Stay Unsealed.
        if matches!(action, Action::Stop) && !bounded {
            self.reconcile_sampling().await;
            self.execution_storage.save_cycle(test_cycle_id.clone()).await?;
            let drafted = self.state_read().cycle_view(&test_cycle_id).is_some_and(|cycle| {
                cycle.test_executions.iter().any(|execution| {
                    execution.saved.is_none()
                        && execution.status == dokime_domain::model::execution::result::ExecutionStatus::Skipped
                        && execution.finished_at.is_none()
                })
            });
            if !drafted {
                return self.seal_settled_cycle(&test_cycle_id).await;
            }
            return Ok(());
        }
        // Persist cycle state separately from execution evidence. During an
        // active request only running captures need a control-boundary flush;
        // rewriting every untouched plan races the provider's final sealing.
        let runtime = self.clone();
        tokio::spawn(async move {
            let result = async {
                if bounded {
                    runtime.execution_storage.save_cycle_header(test_cycle_id.clone()).await?;
                    runtime.reconcile_sampling().await;
                    let cycle =
                        runtime.state_read().cycle_view(&test_cycle_id).ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
                    let ids = cycle
                        .test_executions
                        .iter()
                        .filter(|execution| execution.status == dokime_domain::model::execution::result::ExecutionStatus::Running)
                        .map(|execution| execution.test_execution_id.clone())
                        .collect::<Vec<_>>();
                    runtime.execution_storage.checkpoint_targets(&cycle, &ids).await
                } else {
                    runtime.reconcile_sampling().await;
                    runtime.execution_storage.save_cycle(test_cycle_id.clone()).await
                }
            }
            .await;
            if let Err(error) = result {
                tracing::error!(cycle_id = %test_cycle_id, %error, "cycle control checkpoint failed");
            }
        });
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn admit_control<'a>(action: &Action, evidence: &'a tokio::sync::Mutex<()>) -> Option<tokio::sync::MutexGuard<'a, ()>> {
    if matches!(action, Action::Stop) { None } else { Some(evidence.lock().await) }
}

/* === Tests === */

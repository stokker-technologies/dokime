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
        let permit = self.admit_owned_work().ok_or(Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            runtime.stop_execution_inner(id, context).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    async fn stop_execution_inner(&self, id: TestExecutionId, context: CommandContext) -> Result<(), Error> {
        let admission = self.plugin_admission.lock().await;
        let execution = self.known_execution(&id).await?.ok_or_else(|| Error::not_found("execution_not_found", "execution not found"))?;
        if !self.state_read().executions.contains_key(&id) {
            return if execution.saved.is_some() { Ok(()) } else { Err(Error::Operation("historical execution is not sealed".into())) };
        }
        let cycle_id = execution.test_cycle_id.clone();
        if self.state_read().cycles[&cycle_id].phase == dokime_domain::model::execution::result::CyclePhase::Sealed {
            return Ok(());
        }
        if execution.saved.is_none() && execution.finished_at.is_none() {
            self.freeze_report_inputs(vec![id.clone()]).await?;
        }
        self.commit_with_key(
            context.idempotency_key,
            crate::transactions::ControlExecution {
                test_execution_id: id.clone(),
                action: Action::Stop,
                occurred_at: crate::model::time::now_utc(),
            },
        )
        .await?;

        let request = crate::transactions::ControlExecution::owning_request(&self.state_read(), &id).cloned();
        let owns_task = request.as_ref().is_some_and(|request| {
            self.provider_tasks
                .lock()
                .expect("provider task registry is healthy")
                .iter()
                .any(|task| task.request_id == request.summary.request_id && !task.settled.is_cancelled())
        });
        let queued = !owns_task
            && request.as_ref().is_some_and(|request| request.summary.phase == crate::transactions::provider::request::RunPhase::Queued);
        if queued {
            let request = request.as_ref().expect("queued request");
            self.commit(crate::transactions::StageQueuedStop {
                request_id: request.summary.request_id.clone(),
                at: crate::model::time::now_utc(),
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        }
        drop(admission);

        if !queued && let Some(request) = &request {
            let pending = self
                .provider_tasks
                .lock()
                .expect("provider task registry is healthy")
                .iter()
                .filter(|task| task.request_id == request.summary.request_id)
                .map(|task| task.settled.clone())
                .collect::<Vec<_>>();
            for settled in pending {
                settled.cancelled().await;
            }
        }
        let cycle =
            self.state_read().cycle_view(&cycle_id).ok_or_else(|| Error::not_found("cycle_not_found", "execution cycle not found"))?;
        let execution = cycle
            .test_executions
            .iter()
            .find(|execution| execution.test_execution_id == id)
            .ok_or_else(|| Error::not_found("execution_not_found", "execution not found"))?;
        if execution.saved.is_none() {
            self.finish_saved_execution(&cycle, execution).await?;
        }
        if queued {
            let request = request.expect("queued request");
            let at = self.state_read().run_requests[&request.summary.request_id]
                .completion_at
                .clone()
                .ok_or_else(|| Error::Operation("queued stop has no completion timestamp".into()))?;
            self.commit(crate::transactions::CompleteRun { request_id: request.summary.request_id.clone(), at, unsealed: None })
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
            let summary = self.state_read().run_requests[&request.summary.request_id].summary.clone();
            self.request_storage.update(&request.summary.request_id, |receipt| receipt.summary = summary).await?;
        }
        self.execution_storage.save_cycle_header(cycle_id).await?;
        self.dispatch_next_request().await;
        Ok(())
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

    pub(crate) async fn stop_cycle(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Stop, context).await
    }

    pub async fn seal_cycle(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        let permit = self.admit_owned_work().ok_or(Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            runtime.seal_cycle_inner(test_cycle_id, context).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn seal_cycle_inner(&self, test_cycle_id: TestCycleId, context: CommandContext) -> Result<(), Error> {
        let saved = self.state_read().cycles.get(&test_cycle_id).map(|cycle| cycle.saved.is_some());
        if saved == Some(true) {
            return Ok(());
        }
        if saved.is_none() {
            let historical =
                crate::storage::objects::cycle::read(self.storage(), &test_cycle_id, harmos::storage::ObjectVersion::Latest).await?;
            return match historical {
                Some(cycle) if cycle.value.saved.is_some() => Ok(()),
                _ => Err(Error::not_found("cycle_not_found", "active or sealed cycle not found")),
            };
        }

        // Close Admission Before Waiting For Any Provider Or Storage Work.
        let admission = self.plugin_admission.lock().await;
        let unfinished = self
            .state_read()
            .executions
            .values()
            .filter(|execution| execution.test_cycle_id == test_cycle_id && execution.saved.is_none() && execution.finished_at.is_none())
            .map(|execution| execution.test_execution_id.clone())
            .collect();
        self.freeze_report_inputs(unfinished).await?;
        let at = crate::model::time::now_utc();
        self.commit_with_key(
            context.idempotency_key,
            crate::transactions::BeginCycleSeal { test_cycle_id: test_cycle_id.clone(), at: at.clone() },
        )
        .await?;
        let running = self
            .provider_tasks
            .lock()
            .expect("provider task registry is healthy")
            .iter()
            .filter(|task| !task.settled.is_cancelled())
            .map(|task| task.request_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let queued = self
            .state_read()
            .run_requests
            .values()
            .filter(|request| {
                request.summary.cycle_id == test_cycle_id
                    && request.summary.phase == crate::transactions::provider::request::RunPhase::Queued
                    && request.summary.prepared == 0
                    && !running.contains(&request.summary.request_id)
            })
            .map(|request| request.summary.request_id.clone())
            .collect::<Vec<_>>();
        for request_id in queued {
            self.commit(crate::transactions::StageQueuedStop { request_id, at: at.clone() })
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
        }
        drop(admission);

        // Group Continuation May Register Its Task Before Its Predecessor Settles.
        loop {
            let pending = self
                .provider_tasks
                .lock()
                .expect("provider task registry is healthy")
                .iter()
                .filter(|task| task.cycle == test_cycle_id && !task.settled.is_cancelled())
                .map(|task| task.settled.clone())
                .collect::<Vec<_>>();
            if pending.is_empty() {
                break;
            }
            for settled in pending {
                settled.cancelled().await;
            }
        }
        let cycle = self.state_read().cycle_view(&test_cycle_id).ok_or_else(|| Error::not_found("cycle_not_found", "cycle not found"))?;
        for execution in &cycle.test_executions {
            if execution.saved.is_none() {
                self.finish_saved_execution(&cycle, execution).await?;
            }
        }

        // A Queued Receipt Becomes Terminal Only After Its Selection Is Sealed.
        let finishing = self
            .state_read()
            .run_requests
            .values()
            .filter(|request| request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal())
            .map(|request| (request.summary.request_id.clone(), request.completion_at.clone()))
            .collect::<Vec<_>>();
        for (request_id, at) in finishing {
            let at = at.ok_or_else(|| Error::Operation("provider work did not settle; cycle remains sealing".into()))?;
            self.commit(crate::transactions::CompleteRun { request_id: request_id.clone(), at, unsealed: None })
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
            let summary = self.state_read().run_requests[&request_id].summary.clone();
            self.request_storage.update(&request_id, |receipt| receipt.summary = summary).await?;
        }
        self.dispatch_next_request().await;
        self.seal_settled_cycle(&test_cycle_id).await
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
        let requests = self
            .state_read()
            .run_requests
            .values()
            .filter(|request| request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal())
            .map(|request| request.summary.clone())
            .collect::<Vec<_>>();
        for summary in requests {
            let id = summary.request_id.clone();
            self.request_storage
                .update(&id, |receipt| {
                    receipt.summary.paused = summary.paused;
                    receipt.summary.stop_requested = summary.stop_requested;
                })
                .await
                .map_err(|error| unavailable_rejection(error.to_string()))?;
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

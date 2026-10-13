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
use crate::{CommandContext, Error, RuntimeHandle, unavailable_rejection};
use dokime::core::{TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn pause_execution(
        &self,
        id: TestExecutionId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_execution(id, Action::Pause, context).await
    }

    pub async fn resume_execution(
        &self,
        id: TestExecutionId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_execution(id, Action::Resume, context).await
    }

    pub async fn stop_execution(
        &self,
        id: TestExecutionId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_execution(id, Action::Stop, context).await
    }

    async fn control_execution(
        &self,
        id: TestExecutionId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        let permit = self
            .owned_admissions
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                if self.is_shutting_down() {
                    Error::ShuttingDown
                } else {
                    Error::overload(
                        "execution_control_busy",
                        "execution control admission is at capacity; retry later",
                    )
                }
            })?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            runtime.control_execution_inner(id, action, context).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    async fn control_execution_inner(
        &self,
        id: TestExecutionId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        let _admission = admit_control(&action, &self.evidence_admission).await;
        let transaction = crate::transactions::ControlExecution {
            test_execution_id: id.clone(),
            action,
            occurred_at: Timestamp::now_utc(),
        };
        let (request_id, cycle_id) = {
            let state = self.state_read();
            let request = transaction
                .request(&state)
                .map_err(|error| Error::conflict("execution_control_refused", error))?;
            (
                request.summary.request_id.clone(),
                request.summary.cycle_id.clone(),
            )
        };
        self.commit_with_key(context.idempotency_key, transaction)
            .await?;
        drop(_admission);

        let summary = self.state_read().run_requests[&request_id].summary.clone();
        self.request_storage
            .update(&request_id, 0, |receipt| {
                if !receipt.summary.phase.terminal() {
                    receipt.summary.paused = summary.paused;
                    receipt.summary.stop_requested = summary.stop_requested;
                }
            })
            .await?;
        self.execution_storage
            .save_cycle_header(cycle_id.clone())
            .await?;
        self.reconcile_sampling().await;

        let cycle = self
            .state_read()
            .cycle_view(&cycle_id)
            .ok_or_else(|| Error::not_found("not_found", "execution cycle not found"))?;
        if cycle.test_executions.iter().any(|execution| {
            execution.test_execution_id == id
                && execution.status == crate::execution::ExecutionStatus::Running
        }) {
            self.execution_storage
                .checkpoint_targets(&cycle, &[id])
                .await?;
        }
        Ok(())
    }

    pub async fn pause_cycle(
        &self,
        test_cycle_id: TestCycleId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Pause, context)
            .await
    }

    pub async fn resume_cycle(
        &self,
        test_cycle_id: TestCycleId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Resume, context)
            .await
    }

    pub async fn stop_cycle(
        &self,
        test_cycle_id: TestCycleId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.control_cycle(test_cycle_id, Action::Stop, context)
            .await
    }

    async fn control_cycle(
        &self,
        test_cycle_id: TestCycleId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        let runtime = self.clone();
        // The committed transition owns all of its checkpoint work, including
        // later executions if the caller stops waiting after the first one.
        tokio::spawn(async move {
            runtime
                .control_cycle_inner(test_cycle_id, action, context)
                .await
        })
        .await
        .map_err(|error| unavailable_rejection(error.to_string()))?
    }

    async fn control_cycle_inner(
        &self,
        test_cycle_id: TestCycleId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        // Stop must reach the provider even while an unrelated evidence write
        // or historical deletion holds admission on a slow storage device.
        // The journal still validates and serializes the control transition;
        // the terminal receipt still waits for the final evidence checkpoint.
        let _admission = admit_control(&action, &self.evidence_admission).await;
        let occurred_at = Timestamp::now_utc();
        let transaction = Control {
            test_cycle_id: test_cycle_id.clone(),
            action,
            occurred_at: occurred_at.clone(),
        };
        let committed = self
            .commit_with_key(context.idempotency_key, transaction)
            .await?;
        let bounded = self.state_read().run_requests.values().any(|request| {
            request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal()
        });
        if bounded && matches!(action, Action::Pause | Action::Resume) {
            let scopes = self
                .state_read()
                .executions
                .values()
                .filter(|execution| execution.test_cycle_id == test_cycle_id)
                .filter(|execution| execution.status == crate::execution::ExecutionStatus::Running)
                .map(|execution| execution.scope.clone())
                .collect::<Vec<_>>();
            for scope in scopes {
                let mut request = crate::records::RecordPublishRequest::new(
                    dokime::core::Identifier::new(if matches!(action, Action::Pause) {
                        "test.execution.paused"
                    } else {
                        "test.execution.resumed"
                    })
                    .expect("builtin"),
                    crate::records::RecordSeverity::Info,
                );
                let key = RequestKey::new(format!(
                    "cycle-control-record:{}:{}",
                    committed.position.0,
                    scope.test_execution_id.as_ref().expect("execution scope"),
                ));
                request.scope = Some(scope);
                self.commit_keyed(
                    key,
                    crate::transactions::PublishRecord::host(request, occurred_at.clone()),
                )
                .await
                .map_err(|error| unavailable_rejection(error.to_string()))?;
            }
        }
        let request = self
            .state_read()
            .run_requests
            .values()
            .find(|request| {
                request.summary.cycle_id == test_cycle_id && !request.summary.phase.terminal()
            })
            .map(|request| request.summary.clone());
        if let Some(summary) = request {
            let id = summary.request_id.clone();
            self.request_storage
                .update(&id, 0, |receipt| {
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
                    runtime
                        .execution_storage
                        .save_cycle_header(test_cycle_id.clone())
                        .await?;
                    runtime.reconcile_sampling().await;
                    let cycle = runtime
                        .state_read()
                        .cycle_view(&test_cycle_id)
                        .ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
                    let ids = cycle
                        .test_executions
                        .iter()
                        .filter(|execution| {
                            execution.status == crate::execution::ExecutionStatus::Running
                        })
                        .map(|execution| execution.test_execution_id.clone())
                        .collect::<Vec<_>>();
                    runtime
                        .execution_storage
                        .checkpoint_targets(&cycle, &ids)
                        .await
                } else {
                    runtime.reconcile_sampling().await;
                    runtime
                        .execution_storage
                        .save_cycle(test_cycle_id.clone())
                        .await
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
async fn admit_control<'a>(
    action: &Action,
    evidence: &'a tokio::sync::Mutex<()>,
) -> Option<tokio::sync::MutexGuard<'a, ()>> {
    if matches!(action, Action::Stop) {
        None
    } else {
        Some(evidence.lock().await)
    }
}

/* === Tests === */

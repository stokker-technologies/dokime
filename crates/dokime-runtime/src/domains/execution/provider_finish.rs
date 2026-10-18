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
pub(crate) use crate::transactions::FinishProvider as Finish;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::transactions::provider_finish::prepare_finish_effects;
use crate::{
    RuntimeHandle,
    runtime::error::{Error, Result},
};

/* === Constants === */
const RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn finish_provider_cycle(&self, finish: Finish) -> Result<()> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.finish_provider_cycle_inner(finish).await }).await.map_err(|source| Error::Join { source })?
    }
    async fn finish_provider_cycle_inner(&self, mut finish: Finish) -> Result<()> {
        let request_id = finish.request_id.clone();
        let selected_executions = self.state_read().run_requests[&request_id].selected_execution_ids.clone();
        let cycle_id = finish.test_cycle_id.clone();
        let ids = std::mem::take(&mut finish.started_execution_ids);
        let mut projections = match &mut finish.outcome {
            Ok(outcome) => std::mem::take(&mut outcome.executions)
                .into_iter()
                .map(|execution| (execution.test_execution_id.clone(), execution))
                .collect::<std::collections::BTreeMap<_, _>>(),
            Err(_) => std::collections::BTreeMap::new(),
        };
        // Each Bounded Fold Retains The Current Execution's Returned Evidence.
        if ids.is_empty() {
            self.finish_provider_attempts(finish.clone()).await?;
        }
        let mut early_failure = None;
        for id in &ids {
            let mut part = finish.clone();
            part.started_execution_ids = vec![id.clone()];
            if let Ok(outcome) = &mut part.outcome {
                outcome.executions = projections.remove(id.as_str()).into_iter().collect();
            }
            self.finish_provider_attempts(part).await?;
            let current = self.state_read().cycle_view(&cycle_id).ok_or_else(|| Error::Operation("execution cycle disappeared".into()))?;
            let execution = current
                .test_executions
                .iter()
                .find(|execution| &execution.test_execution_id == id)
                .ok_or_else(|| Error::Operation("finished execution disappeared".into()))?;
            loop {
                match self.finish_saved_execution(&current, execution).await {
                    Ok(()) => break,
                    Err(error) => match retry_after(&error) {
                        Some(wait) => tokio::time::sleep(wait).await,
                        None => {
                            early_failure = Some(error);
                            break;
                        }
                    },
                }
            }
            let summary = self.state_read().run_requests[&request_id].summary.clone();
            loop {
                match self.request_storage.update(&request_id, |receipt| receipt.summary = summary.clone()).await {
                    Ok(()) => break,
                    Err(error) if self.is_shutting_down() => return Err(error),
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                }
            }
        }
        // Every Selected Execution Finishes And Seals Before The Next Group.
        if early_failure.is_none() && self.advance_to_next_group(&request_id, &cycle_id).await {
            return Ok(());
        }
        self.commit(crate::transactions::SettleRun { request_id: request_id.clone(), at: finish.finished_at.clone() })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        let mut complete = crate::transactions::CompleteRun {
            request_id: request_id.clone(),
            at: self.state_read().run_requests[&request_id]
                .completion_at
                .clone()
                .ok_or_else(|| Error::Operation("terminal evidence timestamp is missing".into()))?,
            unsealed: None,
        };
        let cycle = self.state_read().cycle_view(&cycle_id).ok_or_else(|| Error::Operation("report cycle disappeared".into()))?;
        let mut pending = cycle
            .test_executions
            .iter()
            .filter(|execution| {
                selected_executions.contains(&execution.test_execution_id) && execution.saved.is_none() && execution.finished_at.is_some()
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut unsealed = Vec::new();
        let mut cause = early_failure.as_ref().map(|error| unsealed_cause("execution", error));
        while !pending.is_empty() {
            let failures = match self.finish_saved_executions(&cycle, pending.clone()).await {
                Ok(failures) => failures,
                Err(error) if self.is_shutting_down() => return Err(error),
                Err(error) => match retry_after(&error) {
                    Some(wait) => {
                        tracing::debug!(%error, %cycle_id, "final execution saves will retry");
                        tokio::time::sleep(wait).await;
                        continue;
                    }
                    None => {
                        tracing::debug!(%error, %cycle_id, "final execution saves failed");
                        cause.get_or_insert_with(|| unsealed_cause("execution", &error));
                        unsealed.extend(pending.drain(..).map(|execution| execution.test_execution_id));
                        continue;
                    }
                },
            };
            let mut retry = Vec::new();
            let mut wait = None;
            for (id, error) in failures {
                if self.is_shutting_down() {
                    return Err(error);
                }
                tracing::debug!(%error, %cycle_id, execution_id = %id, "final execution save failed");
                match retry_after(&error) {
                    Some(after) => {
                        wait = wait.max(Some(after));
                        retry.push(id);
                    }
                    None => {
                        cause.get_or_insert_with(|| unsealed_cause("execution", &error));
                        unsealed.push(id);
                    }
                }
            }
            pending.retain(|execution| retry.contains(&execution.test_execution_id));
            if let Some(wait) = wait {
                // Retry checkpoint publication only. Provider dispatch and outcome
                // folds are outside this loop and cannot be executed again.
                tokio::time::sleep(wait).await;
            }
        }
        // A Run Seals Only Its Fixed Selection; The Container Remains Active.
        if let Some(cause) = &cause {
            tracing::error!(%cycle_id, %cause, "final execution report could not be sealed");
            complete.unsealed = Some(crate::transactions::Unsealed { cause: cause.clone(), execution_ids: unsealed });
        }
        self.commit(complete).await.map_err(|error| Error::Operation(error.to_string()))?;
        loop {
            match self.execution_storage.save_cycle_header(cycle_id.clone()).await {
                Ok(()) => break,
                Err(error) if self.is_shutting_down() => return Err(error),
                Err(_) => tokio::time::sleep(RETRY_INTERVAL).await,
            }
        }
        // The durable terminal receipt acknowledges both sealed executions and
        // their final cycle row, or the permanent cause they were not sealed.
        // A failed transient save leaves it nonterminal.
        let (summary, interruption) = {
            let state = self.state_read();
            let request = &state.run_requests[&request_id];
            (request.summary.clone(), request.interruption())
        };
        loop {
            match self
                .request_storage
                .update(&request_id, |receipt| {
                    receipt.summary = summary.clone();
                    receipt.interruption = interruption.clone();
                })
                .await
            {
                Ok(()) => break,
                Err(error) if self.is_shutting_down() => return Err(error),
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
        self.dispatch_next_request().await;
        Ok(())
    }
    async fn advance_to_next_group(
        &self, request_id: &dokime_domain::model::identity::CommandId, cycle_id: &dokime_domain::model::identity::TestCycleId,
    ) -> bool {
        {
            let state = self.state_read();
            let Some(request) = state.run_requests.get(request_id) else {
                return false;
            };
            if request.groups.is_empty() {
                return false;
            }
            // Fail-fast is the cycle's policy, and it spans groups: a target
            // whose group failed stops the targets queued behind it.
            let failed = request.summary.failed > 0 || request.failure.is_some();
            let continues = state.cycles.get(cycle_id).is_some_and(|cycle| cycle.continue_on_failure);
            if failed && !continues {
                return false;
            }
        }
        let before = self.state_manager.applied();
        if self
            .commit(crate::transactions::AdvanceRunGroup { request_id: request_id.clone(), at: crate::model::time::now_utc() })
            .await
            .is_err()
        {
            return false;
        }
        let (summary, target) = {
            let state = self.state_read();
            let request = &state.run_requests[request_id];
            (request.summary.clone(), request.target.clone())
        };
        loop {
            match self.request_storage.update(request_id, |receipt| receipt.summary = summary.clone()).await {
                Ok(()) => break,
                Err(error) if self.is_shutting_down() => {
                    tracing::error!(%error, "failed to checkpoint the advanced run request");
                    break;
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
        crate::execution_runtime::provider::ProviderCycleRequest {
            request_id: request_id.to_string(),
            test_cycle_id: cycle_id.to_string(),
            target: serde_json::to_value(target).expect("provider target serializes"),
        }
        .submit(self.clone(), before);
        true
    }

    async fn finish_provider_attempts(&self, finish: Finish) -> Result<()> {
        let execution_ids = finish.started_execution_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let quiescence = self.quiesce_execution_sampling(&execution_ids).await?;
        let effects = prepare_finish_effects(&self.state_read(), &finish);
        let finalize_execution_ids = effects.terminal_execution_ids.clone();
        let admission = match self.prepare_sample_admission(&effects.telemetry).await {
            Ok(admission) => admission.within_quiescence(),
            Err(error) => {
                self.resume_sampling(&quiescence).await;
                return Err(error);
            }
        };
        if let Err(error) = self.state_manager.commit(finish).await {
            drop(admission);
            self.resume_sampling(&quiescence).await;
            return Err(Error::Operation(error.to_string()));
        }
        // The admission token retains telemetry ordering and execution staging
        // ownership through the journal commit. No storage wait occurs under that
        // staging lock; capacity was made before committing Finish.
        // This is process-local admission: crash durability begins at checkpoint.
        if let Err(error) = self.admit_samples(admission).await {
            self.resume_sampling(&quiescence).await;
            return Err(error);
        }
        self.refresh_state().await?;
        for execution_id in finalize_execution_ids {
            Box::pin(self.finalize_execution_aggregate_outputs(&execution_id)).await;
        }
        self.complete_sampling(&quiescence).await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn retry_after(error: &Error) -> Option<std::time::Duration> {
    transient_persistence(error).then_some(RETRY_INTERVAL)
}

fn unsealed_cause(subject: &str, error: &Error) -> String {
    format!("final {subject} report was not sealed: {error}")
}

fn transient_persistence(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(harmos::Error::Backend { kind, .. }) = error.downcast_ref::<harmos::Error>() {
            return *kind == harmos::BackendFailure::Transient;
        }
        current = error.source();
    }
    false
}

/* === Tests === */

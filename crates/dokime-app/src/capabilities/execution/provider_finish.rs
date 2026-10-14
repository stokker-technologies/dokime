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
use harmos::MemorySize;

// Local Imports
use crate::transactions::provider_finish::prepare_finish_effects;
use crate::{
    RuntimeHandle,
    error::{Error, Result},
};

/* === Constants === */
/// How many times one finalization may exhaust its computation budget before
/// that exhaustion is final. A slow or loaded host can overrun the budget once;
/// overrunning it repeatedly means the report cannot be produced here.
const EXHAUSTED_ATTEMPTS: u32 = 3;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn finish_provider_cycle(&self, finish: Finish) -> Result<()> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.finish_provider_cycle_inner(finish).await })
            .await
            .map_err(|source| Error::Join { source })?
    }
    async fn finish_provider_cycle_inner(&self, mut finish: Finish) -> Result<()> {
        let request_id = self
            .state_read()
            .run_requests
            .values()
            .find(|request| {
                request.summary.cycle_id == finish.test_cycle_id
                    && !request.summary.phase.terminal()
            })
            .map(|request| request.summary.request_id.clone());
        let Some(request_id) = request_id else {
            return self.finish_provider_attempts(finish, true).await;
        };
        let selected_execution = self.state_read().run_requests[&request_id]
            .test_execution_id
            .clone();
        let cycle_id = finish.test_cycle_id.clone();
        let ids = std::mem::take(&mut finish.started_execution_ids);
        let mut projections = match &mut finish.outcome {
            Ok(outcome) => std::mem::take(&mut outcome.executions)
                .into_iter()
                .map(|execution| (execution.test_execution_id.clone(), execution))
                .collect::<std::collections::BTreeMap<_, _>>(),
            Err(_) => std::collections::BTreeMap::new(),
        };
        // Each bounded fold retains its original returned evidence. The provider
        // still owns scheduling and is invoked once for the entire cycle.
        if ids.is_empty() {
            self.finish_provider_attempts(finish.clone(), false).await?;
        }
        for id in &ids {
            let mut part = finish.clone();
            part.started_execution_ids = vec![id.clone()];
            if let Ok(outcome) = &mut part.outcome {
                outcome.executions = projections.remove(id.as_str()).into_iter().collect();
            }
            self.finish_provider_attempts(part, false).await?;
            let summary = self.state_read().run_requests[&request_id].summary.clone();
            loop {
                match self
                    .request_storage
                    .update(
                        &request_id,
                        harmos::MemorySize::memory_size(&summary),
                        |receipt| receipt.summary = summary.clone(),
                    )
                    .await
                {
                    Ok(()) => break,
                    Err(error) if self.is_shutting_down() => return Err(error),
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                }
            }
        }
        // A cycle whose cases span several provider execution targets runs one
        // group per target, sequentially, under this one request. The run only
        // settles once the last group has finished or the policy stops it.
        if self.advance_to_next_group(&request_id, &cycle_id).await {
            return Ok(());
        }
        let settled = self.state_read().run_requests[&request_id]
            .settled_execution_ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        self.commit(crate::transactions::SettleRun {
            request_id: request_id.clone(),
            at: finish.finished_at.clone(),
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?;
        let mut complete = crate::transactions::CompleteRun {
            request_id: request_id.clone(),
            at: self.state_read().run_requests[&request_id]
                .completion_at
                .clone()
                .ok_or_else(|| Error::Operation("terminal evidence timestamp is missing".into()))?,
            sealed_cycle: None,
            sealed_executions: Vec::new(),
            unsealed: None,
        };
        let (summary, cycle) = {
            use harmos::state::Transaction;
            let mut preview = self.state_read().as_ref().clone();
            complete.check(&preview).map_err(Error::Operation)?;
            complete.apply(&mut preview);
            (
                preview.run_requests[&request_id].summary.clone(),
                preview.cycle_view(&cycle_id).expect("request cycle exists"),
            )
        };
        let mut pending = cycle
            .test_executions
            .iter()
            .filter(|execution| {
                selected_execution
                    .as_ref()
                    .is_none_or(|id| id == &execution.test_execution_id)
                    && (ids.contains(&execution.test_execution_id)
                        || settled.contains(&execution.test_execution_id)
                        || execution.status == crate::execution::ExecutionStatus::Skipped)
            })
            .cloned()
            .collect::<Vec<_>>();
        // An earlier attempt whose report was never produced is still a cycle
        // member; seal it now so a rerun can seal the cycle.
        pending.extend(self.unsealed_members(&cycle, &pending).await);
        let mut unsealed = Vec::new();
        let mut cause = None;
        let mut exhausted = 0;
        let mut exhausted_members = std::collections::BTreeMap::new();
        while !pending.is_empty() {
            let failures = match self.finish_saved_executions(&cycle, pending.clone()).await {
                Ok(failures) => failures,
                Err(error) if self.is_shutting_down() => return Err(error),
                Err(error) if is_final(&error, &mut exhausted) => {
                    tracing::debug!(%error, %cycle_id, "final execution saves failed");
                    cause.get_or_insert_with(|| {
                        format!("final execution report was not sealed: {error}")
                    });
                    unsealed.extend(
                        pending
                            .drain(..)
                            .map(|execution| execution.test_execution_id),
                    );
                    continue;
                }
                Err(error) => {
                    tracing::debug!(%error, %cycle_id, "final execution saves will retry");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
            };
            let mut retry = Vec::new();
            for (id, error) in failures {
                if self.is_shutting_down() {
                    return Err(error);
                }
                tracing::debug!(%error, %cycle_id, execution_id = %id, "final execution save failed");
                if is_final(&error, exhausted_members.entry(id.clone()).or_default()) {
                    cause.get_or_insert_with(|| {
                        format!("final execution report was not sealed: {error}")
                    });
                    unsealed.push(id);
                } else {
                    retry.push(id);
                }
            }
            pending.retain(|execution| retry.contains(&execution.test_execution_id));
            if !pending.is_empty() {
                // Retry checkpoint publication only. Provider dispatch and outcome
                // folds are outside this loop and cannot be executed again.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        let terminal = matches!(
            cycle.lifecycle,
            dokime::execution::CycleStatus::Completed
                | dokime::execution::CycleStatus::Failed
                | dokime::execution::CycleStatus::Blocked
                | dokime::execution::CycleStatus::Stopped
        );
        // A cycle whose members are not all sealed can never be sealed itself.
        let sealed_cycle = if terminal && cause.is_none() {
            let mut exhausted = 0;
            loop {
                match self.finish_saved_cycle(&cycle).await {
                    Ok(cycle) => break Some(cycle),
                    Err(error) if self.is_shutting_down() => return Err(error),
                    Err(error) if is_final(&error, &mut exhausted) => {
                        tracing::debug!(%error, %cycle_id, "final cycle save failed");
                        cause = Some(format!("final cycle report was not sealed: {error}"));
                        break None;
                    }
                    Err(error) => {
                        tracing::debug!(%error, %cycle_id, "final cycle save will retry");
                        if computation_exhausted(&error) {
                            self.execution_storage.restart_cycle_report(&cycle_id).await;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        } else {
            None
        };
        if let Some(sealed_cycle) = sealed_cycle {
            let bytes = sealed_cycle
                .memory_size()
                .saturating_add(std::mem::size_of::<crate::transactions::CompleteRun>());
            let _change = self
                .storage()
                .prepare_production(std::num::NonZeroUsize::new(bytes).unwrap())?;
            complete.sealed_cycle = Some(sealed_cycle.header.clone());
            complete.sealed_executions = sealed_cycle.executions.clone();
            self.commit(complete)
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
            self.execution_storage.retire_cycle_report(&cycle_id).await;
        } else {
            // A permanent failure settles the run Failed with its cause. The
            // cycle is terminal and unsealed; nothing claims a report that was
            // never produced, and the next finalization starts afresh.
            if let Some(cause) = &cause {
                tracing::error!(%cycle_id, %cause, "final report could not be sealed");
                let mut released = Vec::with_capacity(unsealed.len());
                for id in unsealed {
                    if self.execution_storage.release_finalization(&id).await {
                        released.push(id);
                    }
                }
                complete.unsealed = Some(crate::transactions::Unsealed {
                    cause: cause.clone(),
                    execution_ids: released,
                });
                self.execution_storage.retire_cycle_report(&cycle_id).await;
            }
            self.commit(complete)
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
            loop {
                match self
                    .execution_storage
                    .save_cycle_header(cycle_id.clone())
                    .await
                {
                    Ok(()) => break,
                    Err(error) if self.is_shutting_down() => return Err(error),
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                }
            }
        }
        // The durable terminal receipt acknowledges both sealed executions and
        // their final cycle row, or the permanent cause they were not sealed.
        // A failed transient save leaves it nonterminal.
        let summary = if cause.is_some() {
            self.state_read().run_requests[&request_id].summary.clone()
        } else {
            summary
        };
        let growth = harmos::MemorySize::memory_size(&summary)
            .saturating_add(cause.as_ref().map_or(0, String::len));
        loop {
            match self
                .request_storage
                .update(&request_id, growth, |receipt| {
                    receipt.summary = summary.clone();
                    if cause.is_some() {
                        receipt.interruption = cause.clone();
                    }
                })
                .await
            {
                Ok(()) => break,
                Err(error) if self.is_shutting_down() => return Err(error),
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
        Ok(())
    }
    async fn advance_to_next_group(
        &self,
        request_id: &dokime::core::CommandId,
        cycle_id: &dokime::core::TestCycleId,
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
            let continues = state
                .cycles
                .get(cycle_id)
                .is_some_and(|cycle| cycle.continue_on_failure);
            if failed && !continues {
                return false;
            }
        }
        let before = self.state_manager.applied();
        if self
            .commit(crate::transactions::AdvanceRunGroup {
                request_id: request_id.clone(),
                at: dokime::core::Timestamp::now_utc(),
            })
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
            match self
                .request_storage
                .update(
                    request_id,
                    harmos::MemorySize::memory_size(&summary),
                    |receipt| receipt.summary = summary.clone(),
                )
                .await
            {
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

    async fn finish_provider_attempts(&self, finish: Finish, finalize_cycle: bool) -> Result<()> {
        let cycle_id = finish.test_cycle_id.clone();
        let execution_ids = finish
            .started_execution_ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let quiescence = self.quiesce_execution_sampling(&execution_ids).await?;
        let effects = prepare_finish_effects(&self.state_read(), &finish);
        let finalize_execution_ids = effects.terminal_execution_ids.clone();
        let admission = match self.prepare_sample_admission(&effects.telemetry).await {
            Ok(admission) => admission,
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
        let completed = {
            let state = self.state_read();
            state
                .executions
                .values()
                .filter(|execution| execution_ids.contains(&execution.test_execution_id))
                .filter_map(|execution| {
                    state
                        .cycle_view(&execution.test_cycle_id)
                        .map(|cycle| (cycle, execution.clone()))
                })
                .collect::<Vec<_>>()
        };
        let mut storage_error = None;
        for (cycle, execution) in completed.into_iter().filter(|_| finalize_cycle) {
            if let Err(error) = self.finish_saved_execution(&cycle, &execution).await {
                storage_error.get_or_insert(error);
            }
        }
        self.complete_sampling(&quiescence).await;
        storage_error.map_or(Ok(()), Err)?;
        if finalize_cycle {
            self.execution_storage.save_cycle(cycle_id).await?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
/// Whether a finalization failure is final. An exhausted computation budget is
/// transient until it recurs `EXHAUSTED_ATTEMPTS` times for the same save.
fn is_final(error: &Error, exhausted: &mut u32) -> bool {
    if !computation_exhausted(error) {
        return cannot_retry_save(error);
    }
    *exhausted += 1;
    *exhausted >= EXHAUSTED_ATTEMPTS
}

fn computation_exhausted(error: &Error) -> bool {
    matches!(
        error,
        Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "query computation time"
        }) | Error::Storage(dokime::storage::Error::Runtime(
            harmos::Error::ResourceExhausted {
                resource: "query computation time"
            }
        ))
    )
}

fn cannot_retry_save(error: &Error) -> bool {
    let source = match error {
        Error::Harmos(source) | Error::Storage(dokime::storage::Error::Runtime(source)) => source,
        Error::InvalidReportInputs(_) | Error::Storage(_) => return true,
        _ => return false,
    };
    // Shared Read Admission Can Be Refused While Other Captures Own Capacity.
    // Retry The Original Finalization; Its Own Size And Computation Limits Remain Final.
    if matches!(
        source,
        harmos::Error::ResourceExhausted {
            resource: "storage views" | "storage read memory"
        }
    ) {
        return false;
    }
    matches!(
        source,
        harmos::Error::ResourceExhausted { .. }
            | harmos::Error::ViewExpired
            | harmos::Error::InvalidInput { .. }
            | harmos::Error::DifferentHistory
            | harmos::Error::Gap { .. }
            | harmos::Error::Backend {
                kind: harmos::BackendFailure::Permanent
                    | harmos::BackendFailure::Corrupt
                    | harmos::BackendFailure::Conflict
                    | harmos::BackendFailure::Unsupported,
                ..
            }
    )
}

/* === Tests === */

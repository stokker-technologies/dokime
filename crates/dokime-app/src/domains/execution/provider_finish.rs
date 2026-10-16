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
/// The wait before retrying any other transient finalization failure.
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
        let cycle = {
            use harmos::state::Transaction;
            let state = self.state_read();
            let request = state
                .run_requests
                .get(&request_id)
                .ok_or_else(|| Error::Operation("report request disappeared".into()))?;
            let header = state
                .cycles
                .get(&cycle_id)
                .ok_or_else(|| Error::Operation("report cycle disappeared".into()))?;
            let members = header.execution_ids.iter().try_fold(0usize, |bytes, id| {
                let member = state
                    .executions
                    .get(id)
                    .ok_or_else(|| Error::Operation("report member disappeared".into()))?;
                bytes
                    .checked_add(id.memory_size())
                    .and_then(|bytes| bytes.checked_add(member.memory_size()))
                    .ok_or_else(|| Error::Operation("report preview memory overflows".into()))
            })?;
            // Cover The Focused State, Its Lifecycle Mutations And The Extracted Cycle Together.
            let bytes = std::mem::size_of::<crate::state::RuntimeState>()
                .saturating_add(uuid::fmt::Hyphenated::LENGTH)
                .saturating_add("1970-01-01T00:00:00Z".len())
                .saturating_add(state.session_id.memory_size().saturating_mul(2))
                .saturating_add(harmos::btree_map_memory::<
                    dokime::core::CommandId,
                    crate::transactions::provider::request::RunRequest,
                >(1))
                .saturating_add(request.memory_size())
                .saturating_add(request_id.memory_size())
                .saturating_add(harmos::btree_map_memory::<
                    dokime::core::TestCycleId,
                    dokime::execution::TestCycle,
                >(1))
                .saturating_add(harmos::btree_map_memory::<
                    dokime::core::TestExecutionId,
                    crate::execution::TestExecution,
                >(header.execution_ids.len()))
                .saturating_add(header.memory_size().saturating_mul(3))
                .saturating_add(members.saturating_mul(3));
            let limit = std::num::NonZeroUsize::new(bytes)
                .filter(|bytes| bytes.get() != usize::MAX)
                .ok_or_else(|| Error::Operation("report preview memory overflows".into()))?;
            let memory = self.storage().prepare_production(limit)?;
            let mut preview = crate::state::RuntimeState::new(state.session_id.clone());
            preview
                .run_requests
                .insert(request_id.clone(), request.clone());
            preview.cycles.insert(cycle_id.clone(), header.clone());
            for id in &header.execution_ids {
                preview
                    .executions
                    .insert(id.clone(), state.executions[id].clone());
            }
            complete.check(&preview).map_err(Error::Operation)?;
            complete.apply(&mut preview);
            let cycle = preview
                .cycle_view(&cycle_id)
                .ok_or_else(|| Error::Operation("report cycle disappeared".into()))?;
            drop(preview);
            memory.finish(cycle)?.try_into_value()?
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
                        unsealed.extend(
                            pending
                                .drain(..)
                                .map(|execution| execution.test_execution_id),
                        );
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
        let terminal = matches!(
            cycle.lifecycle,
            dokime::execution::CycleStatus::Completed
                | dokime::execution::CycleStatus::Failed
                | dokime::execution::CycleStatus::Blocked
                | dokime::execution::CycleStatus::Stopped
        );
        // A cycle whose members are not all sealed can never be sealed itself.
        let sealed_cycle = if terminal && cause.is_none() {
            let operation = self.operation_progress().begin("Finalizing Cycle");
            let mut attempt = 1;
            loop {
                match self.finish_saved_cycle(&cycle, &operation).await {
                    Ok(cycle) => {
                        operation.finish();
                        break Some(cycle);
                    }
                    Err(error) if self.is_shutting_down() => {
                        operation.fail(&error);
                        return Err(error);
                    }
                    Err(error) => match retry_after(&error) {
                        Some(wait) => {
                            tracing::debug!(%error, %cycle_id, "final cycle save will retry");
                            attempt += 1;
                            operation.phase(format!("Retrying (Attempt {attempt})"));
                            operation.indeterminate();
                            tokio::time::sleep(wait).await;
                        }
                        None => {
                            tracing::debug!(%error, %cycle_id, "final cycle save failed");
                            operation.fail(&error);
                            cause = Some(unsealed_cause("cycle", &error));
                            break None;
                        }
                    },
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
            // Report publication failure leaves the frozen inputs available
            // for retry. It does not change the recorded test outcome.
            if let Some(cause) = &cause {
                tracing::error!(%cycle_id, %cause, "final report could not be sealed");
                complete.unsealed = Some(crate::transactions::Unsealed {
                    cause: cause.clone(),
                    execution_ids: unsealed,
                });
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
        let (summary, interruption) = {
            let state = self.state_read();
            let request = &state.run_requests[&request_id];
            (request.summary.clone(), request.interruption())
        };
        let growth = harmos::MemorySize::memory_size(&summary)
            .saturating_add(interruption.as_ref().map_or(0, String::len));
        loop {
            match self
                .request_storage
                .update(&request_id, growth, |receipt| {
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
/// Retry Transient Persistence Failures At The Original Evidence Boundary.
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

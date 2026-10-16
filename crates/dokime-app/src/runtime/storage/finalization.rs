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
use harmos::MemorySize;
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct PendingRun {
    request_id: dokime::core::CommandId,
    at: Timestamp,
}
impl MemorySize for PendingRun {
    fn memory_size(&self) -> usize {
        self.request_id
            .memory_size()
            .saturating_add(self.at.memory_size())
    }
}
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl RuntimeHandle {
    /// Resume Publication Without Dispatching Providers Or Changing Frozen Outcomes.
    pub(crate) async fn retry_pending_reports(&self) -> Result<()> {
        let state = self.state_read();
        let bytes = state.run_requests.iter().fold(
            harmos::btree_map_memory::<dokime::core::TestCycleId, PendingRun>(
                state.run_requests.len(),
            ),
            |bytes, (id, request)| {
                bytes
                    .saturating_add(id.memory_size())
                    .saturating_add(request.summary.cycle_id.memory_size())
                    .saturating_add(request.completion_at.memory_size())
            },
        );
        let selected = self
            .storage()
            .produce(bound(bytes)?, |_| {
                let mut selected = std::collections::BTreeMap::new();
                for (id, request) in &state.run_requests {
                    let Some(at) = &request.completion_at else {
                        continue;
                    };
                    if !state.cycles.contains_key(&request.summary.cycle_id) {
                        continue;
                    }
                    let newer = selected
                        .get(&request.summary.cycle_id)
                        .is_none_or(|previous: &PendingRun| previous.at.as_str() < at.as_str());
                    if newer {
                        selected.insert(
                            request.summary.cycle_id.clone(),
                            PendingRun {
                                request_id: id.clone(),
                                at: at.clone(),
                            },
                        );
                    }
                }
                Ok(selected)
            })?
            .try_into_value()?;
        drop(state);
        for (cycle_id, pending) in selected.iter() {
            let PendingRun { request_id, at } = pending;
            if self
                .state_read()
                .cycles
                .get(cycle_id)
                .is_some_and(|cycle| cycle.saved.is_some())
            {
                self.sync_report_receipt(request_id).await?;
                self.execution_storage.retire_cycle_report(cycle_id).await;
                continue;
            }
            let complete = crate::transactions::CompleteRun {
                request_id: request_id.clone(),
                at: at.clone(),
                sealed_cycle: None,
                sealed_executions: Vec::new(),
                unsealed: None,
            };
            let cycle = {
                use harmos::state::Transaction;
                let state = self.state_read();
                let request = state
                    .run_requests
                    .get(request_id)
                    .ok_or_else(|| Error::Operation("report request disappeared".into()))?;
                let header = state
                    .cycles
                    .get(cycle_id)
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
                    .cycle_view(cycle_id)
                    .ok_or_else(|| Error::Operation("report cycle disappeared".into()))?;
                drop(preview);
                memory.finish(cycle)?.try_into_value()?
            };
            if !matches!(
                cycle.lifecycle,
                dokime::execution::CycleStatus::Completed
                    | dokime::execution::CycleStatus::Failed
                    | dokime::execution::CycleStatus::Blocked
                    | dokime::execution::CycleStatus::Stopped
            ) {
                continue;
            }
            for execution in &cycle.test_executions {
                if execution.saved.is_none() {
                    self.finish_saved_execution(&cycle, execution).await?;
                }
            }
            let progress = self
                .operation_progress()
                .begin("Retrying Report Publication");
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
            self.commit(crate::transactions::CompleteRun {
                sealed_cycle: Some(sealed.header.clone()),
                sealed_executions: sealed.executions.clone(),
                ..complete
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
            self.sync_report_receipt(request_id).await?;
            self.execution_storage.retire_cycle_report(cycle_id).await;
        }
        Ok(())
    }

    async fn sync_report_receipt(&self, request_id: &dokime::core::CommandId) -> Result<()> {
        let (summary, interruption) = {
            let state = self.state_read();
            let request = state
                .run_requests
                .get(request_id)
                .ok_or_else(|| invalid("retried run request disappeared"))?;
            (request.summary.clone(), request.interruption())
        };
        let bytes = summary
            .memory_size()
            .saturating_add(interruption.memory_size());
        self.request_storage
            .update(request_id, bytes, |receipt| {
                receipt.summary = summary.clone();
                receipt.interruption = interruption.clone();
            })
            .await?;
        Ok(())
    }

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
        let view = self
            .storage()
            .view_with_limits(self.config().storage.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let latest = ExecutionView::open(
            self.storage(),
            view,
            id,
            harmos::storage::ObjectVersion::Latest,
            limit,
        )
        .await?;
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
        self.execution_storage.retry_final_report(id).await?;
        Ok(())
    }
}
/* === Public Functions === */
/* === Private Functions === */
fn bound(bytes: usize) -> Result<std::num::NonZeroUsize> {
    std::num::NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(|| invalid("report retry selection memory overflows"))
}
fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}
/* === Tests === */

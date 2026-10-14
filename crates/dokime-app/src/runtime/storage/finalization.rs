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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
// Third Party Imports
use futures_util::{StreamExt, stream::FuturesUnordered};
use harmos::{MemorySize, storage::StorageProductionPreparation};
// Local Imports
use super::{execution::ExecutionView, executions::CaptureInputs};
use crate::{
    Error, Result, RuntimeHandle,
    execution::{CycleView, TestExecution, context::source::CapturedSource},
};
use dokime::{core::TestExecutionId, execution::ExecutionStatus};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct FinalizationOrder {
    source_bytes: usize,
    _guard: tokio::sync::OwnedMutexGuard<()>,
    _memory: StorageProductionPreparation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// Seals each execution's final report and returns those that failed,
    /// each with its own cause. One execution's failure never stops its
    /// siblings from sealing.
    pub(crate) async fn finish_saved_executions(
        &self,
        cycle: &CycleView,
        executions: Vec<TestExecution>,
    ) -> Result<Vec<(TestExecutionId, Error)>> {
        let memory = self.storage().prepare_production(bound(
            std::mem::size_of::<FinalizationOrder>() + 2 * std::mem::size_of::<usize>(),
        )?)?;
        let order = Arc::new(FinalizationOrder {
            source_bytes: source_header_bytes(cycle)?,
            _guard: self.execution_storage.cycle_save.clone().lock_owned().await,
            _memory: memory,
        });
        // Keep Four Independent Captures Active Until The Original Set Settles.
        let Some(first) = executions.first() else {
            return Ok(Vec::new());
        };
        let capture = self.finish_saved_execution_attributed(cycle, first, order.clone());
        let capture_bytes = std::mem::size_of_val(&capture)
            .checked_add(256)
            .and_then(|bytes| bytes.checked_mul(4))
            .ok_or_else(|| invalid("terminal capture pool memory overflows"))?;
        let _captures = self.storage().prepare_production(bound(capture_bytes)?)?;
        drop(capture);
        let mut failures = Vec::new();
        let mut pending = executions.iter().collect::<Vec<_>>();
        let mut concurrency = 4;
        loop {
            let mut remaining = pending.iter();
            let mut active = FuturesUnordered::new();
            for execution in remaining.by_ref().take(concurrency) {
                active.push(self.finish_saved_execution_attributed(
                    cycle,
                    execution,
                    order.clone(),
                ));
            }
            let mut refused = Vec::new();
            while let Some((execution, result)) = active.next().await {
                match result {
                    Ok(()) => {}
                    // Large Reports May Fit Individually But Not As A Parallel Group.
                    Err(error)
                        if concurrency > 1 && super::record_capture::read_capacity(&error) =>
                    {
                        refused.push(execution);
                    }
                    Err(error) => failures.push((execution.test_execution_id.clone(), error)),
                }
                if let Some(execution) = remaining.next() {
                    active.push(self.finish_saved_execution_attributed(
                        cycle,
                        execution,
                        order.clone(),
                    ));
                }
            }
            if refused.is_empty() {
                return Ok(failures);
            }
            concurrency = 1;
            pending = refused;
        }
    }

    /// Earlier settled attempts of this cycle whose final report was never
    /// produced. The cycle seal still requires them, so the next finalization
    /// seals them too. Attempts a restart interrupted are not sealed here. A
    /// member whose seal cannot be read is left to the cycle seal, which
    /// reports it.
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
            .view_with_limits(self.config().storage_reads.execution)
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
        order: Arc<FinalizationOrder>,
    ) -> (&'a TestExecution, Result<()>) {
        let result = self
            .finish_saved_execution_ordered(cycle, execution, order)
            .await;
        (execution, result)
    }

    pub(crate) async fn finish_saved_execution(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<()> {
        let memory = self.storage().prepare_production(bound(
            std::mem::size_of::<FinalizationOrder>() + 2 * std::mem::size_of::<usize>(),
        )?)?;
        let order = Arc::new(FinalizationOrder {
            source_bytes: source_header_bytes(cycle)?,
            _guard: self.execution_storage.cycle_save.clone().lock_owned().await,
            _memory: memory,
        });
        self.finish_saved_execution_ordered(cycle, execution, order)
            .await
    }

    async fn finish_saved_execution_ordered(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
        order: Arc<FinalizationOrder>,
    ) -> Result<()> {
        let bytes = order
            .source_bytes
            .checked_add(execution.memory_size())
            .filter(|bytes| *bytes != usize::MAX)
            .ok_or_else(|| invalid("terminal source memory overflows"))?;
        let source = self
            .storage()
            .produce(bound(bytes)?, |_| {
                Ok(CycleView {
                    saved: cycle.saved.clone(),
                    test_cycle_id: cycle.test_cycle_id.clone(),
                    session_id: cycle.session_id.clone(),
                    metadata: cycle.metadata.clone(),
                    lifecycle: cycle.lifecycle,
                    created_at: cycle.created_at.clone(),
                    started_at: cycle.started_at.clone(),
                    updated_at: cycle.updated_at.clone(),
                    test_executions: vec![execution.clone()],
                    records: cycle.records.clone(),
                    continue_on_failure: cycle.continue_on_failure,
                    result: cycle.result.clone(),
                })
            })?
            .try_into_value()?;
        let id = execution.test_execution_id.clone();
        let runtime = self.clone();
        tokio::spawn(async move {
            let _order = order;
            if runtime.execution_storage.has_final_report(&id).await {
                runtime.execution_storage.retry_final_report(&id).await?;
                return Ok(());
            }
            let execution = source
                .test_executions
                .iter()
                .find(|execution| execution.test_execution_id == id)
                .ok_or_else(|| invalid("terminal execution is absent from its cycle"))?;
            // Skipped Executions Have No Producers, But Still Close Their Admission.
            if runtime.execution_storage.acquisition(&id).await.is_none() {
                runtime
                    .execution_storage
                    .acquire(&source, execution)
                    .await?;
            }
            runtime.execution_storage.finish_outputs(&id).await?;
            let inputs = runtime
                .execution_storage
                .prepare_capture(&source, execution, true)
                .await?;
            let inspection = runtime.capture_execution_view(&source, &inputs).await?;
            let view = inspection.view();
            let limit = view.decoded_bytes()?;

            let outputs = view.project_shared(&inputs.outputs, limit, |outputs, allowance| {
                check(outputs.memory_size(), allowance)?;
                Ok(outputs.clone())
            })?;
            let prepared = runtime
                .prepare_final_execution_report(&inspection, outputs)
                .await?;
            let omissions = prepared.captured_input().input.omissions.clone();
            runtime
                .execution_storage
                .publish_final_report(inspection, prepared, dokime::core::Timestamp::now_utc())
                .await?;
            if !omissions.is_empty() {
                // Recorded once the cycle settles: a record admitted while
                // the cycle is still being sealed would change what it seals.
                runtime
                    .execution_storage
                    .report_omissions
                    .lock()
                    .await
                    .entry(execution.test_cycle_id.clone())
                    .or_default()
                    .push((execution.test_execution_id.clone(), omissions));
            }
            Ok(())
        })
        .await?
    }
    pub(super) async fn capture_execution_view(
        &self,
        source: &CycleView,
        inputs: &CaptureInputs,
    ) -> Result<ExecutionView> {
        // Pin The Saved Revision And Every Frozen Output Before Reading Evidence.
        let through = &*inputs.through;
        if !through.covers(&inputs.revision.position)? {
            return Err(invalid("report evidence precedes its captured revision"));
        }
        for output in inputs.outputs.iter() {
            if !through.covers(&output.file.position)? {
                return Err(invalid("report evidence precedes its captured output"));
            }
        }
        self.storage().flush_until(through).await?;
        let view = self
            .storage()
            .view_with_limits(self.config().storage_reads.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let captured_at = dokime::core::Timestamp::now_utc();
        let capture_work = view.begin_computation()?;
        let preparation = self.storage().prepare_production(limit)?;
        let (mut context, logo, owner_node_id, record_contracts, telemetry_contracts) = {
            let state = self.state_read();
            let brand = self.execution_storage.brand.read().unwrap();
            let captured = super::presentation::capture_prepared(
                &state,
                source,
                &inputs.execution,
                brand.as_ref(),
                None,
            )?;
            let records = state.historical_record_contracts();
            let telemetry = &state.telemetry_definitions;
            let telemetry_bytes = telemetry.values().fold(
                harmos::btree_map_memory::<
                    dokime::core::Identifier,
                    BTreeMap<u32, dokime::telemetry::TelemetryContract>,
                >(telemetry.len()),
                |bytes, contract| {
                    bytes
                        .saturating_add(contract.memory_size())
                        .saturating_add(contract.stream.memory_size())
                        .saturating_add(harmos::btree_map_memory::<
                            u32,
                            dokime::telemetry::TelemetryContract,
                        >(1))
                },
            );
            check(
                records
                    .memory_size()
                    .saturating_add(telemetry_bytes)
                    .saturating_add(captured.context.memory_size())
                    .saturating_add(inputs.revision.memory_size())
                    .saturating_add(inputs.outputs.memory_size())
                    .saturating_add(captured_at.memory_size()),
                limit,
            )?;
            let telemetry = telemetry
                .values()
                .map(|contract| {
                    (
                        contract.stream.clone(),
                        BTreeMap::from([(contract.schema_version, contract.clone())]),
                    )
                })
                .collect();
            (
                captured.context,
                captured.logo,
                state.identity.node.node_id.clone(),
                records.clone(),
                telemetry,
            )
        };
        // A plot nobody configured draws with the application defaults, frozen
        // here so a report rendered again from this capture draws the same.
        let defaults = self.settings().read(|document| document.report);
        for document in context.execution_documents.values_mut() {
            defaults.apply(document);
        }
        if let Some(asset) = logo {
            let captured = super::presentation::capture_logo(self.storage(), &asset).await?;
            if let Some(logo) = &mut context.appearance.logo {
                logo.url = (*captured).clone();
            }
        }
        let captured = preparation
            .finish(CapturedSource {
                captured_at,
                owner_node_id: owner_node_id.clone(),
                execution: (*inputs.revision).clone(),
                evidence_through: through.clone(),
                outputs: (*inputs.outputs).clone(),
                context,
                record_contracts,
                telemetry_contracts,
            })?
            .try_into_value()?
            .try_into_shared()?;
        drop(capture_work);
        let inspection = ExecutionView::captured(
            self.storage(),
            view.clone(),
            self.config().storage_reads.execution,
            captured.clone(),
            &owner_node_id,
            limit,
        )
        .await?;
        Ok(inspection)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn source_header_bytes(cycle: &CycleView) -> Result<usize> {
    let members = cycle
        .test_executions
        .iter()
        .try_fold(0usize, |bytes, member| {
            bytes
                .checked_add(member.memory_size())
                .ok_or_else(|| invalid("terminal members memory overflows"))
        })?;
    cycle
        .memory_size()
        .checked_sub(members)
        .filter(|bytes| *bytes != usize::MAX)
        .ok_or_else(|| invalid("terminal source memory overflows"))
}

fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes).ok_or_else(|| invalid("terminal source memory overflows"))
}
fn check(bytes: usize, allowance: NonZeroUsize) -> harmos::Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        tracing::debug!(
            requested = bytes,
            allowed = allowance.get(),
            "terminal report allocation exceeds allowance"
        );
        Err(exhausted())
    } else {
        Ok(())
    }
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "final report materialization memory",
    }
}
fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */

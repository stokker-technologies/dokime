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
use super::executions::{CaptureInputs, ExecutionStorage};
use crate::execution::context::source::CapturedSource;
use std::{collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectVersion, QueryValue, StorageProductionPreparation},
};

// Local Imports
use super::execution::ExecutionView;
use crate::{
    Error, Result, RuntimeHandle,
    execution::{CycleView, TestExecution},
};
use dokime::core::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn execution_snapshot(
        &self,
        execution: &dokime::core::TestExecutionId,
    ) -> Result<ExecutionView> {
        let identity = self
            .storage()
            .produce(bound(execution.memory_size())?, |_| Ok(execution.clone()))?
            .try_into_value()?;
        let runtime = self.clone();
        let operation = async move {
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let source = runtime.snapshot_source(&identity).await?;
            let execution = source
                .test_executions
                .iter()
                .find(|member| member.test_execution_id == *identity)
                .ok_or_else(|| invalid("snapshot execution is outside its cycle"))?;
            runtime.snapshot_member(&source, execution).await
        };
        let bytes = std::mem::size_of_val(&operation)
            .checked_add(std::mem::size_of::<StorageProductionPreparation>())
            .ok_or_else(|| invalid("report capture task memory overflows"))?;
        let memory = self.storage().prepare_production(bound(bytes)?)?;
        tokio::spawn(async move {
            let _memory = memory;
            operation.await
        })
        .await?
    }

    async fn snapshot_source(&self, execution: &TestExecutionId) -> Result<QueryValue<CycleView>> {
        self.execution_storage
            .report_source(execution, &self.config().storage)
            .await
    }

    pub(super) async fn snapshot_member(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<ExecutionView> {
        // Finalization Captures Meet A Full Ledger With Their Own Back-Pressure.
        let view = self
            .storage()
            .view_with_limits(self.config().storage.interactive)
            .await?;
        let limit = view.decoded_bytes()?;
        if let Some(inspection) = ExecutionView::open(
            self.storage(),
            view,
            &execution.test_execution_id,
            ObjectVersion::Latest,
            limit,
        )
        .await?
            && inspection.execution().saved.is_some()
        {
            if execution.saved.is_some() && inspection.execution() != execution {
                return Err(invalid("saved report member differs from frozen cycle"));
            }
            if execution.saved.is_none() {
                let expected = inspection.view().project_shared(
                    inspection.revision()?,
                    bound(inspection.execution().memory_size())?,
                    |revision, _| {
                        let mut execution = revision.value.clone();
                        execution.saved = None;
                        Ok(execution)
                    },
                )?;
                if &*expected != execution {
                    return Err(invalid("saved snapshot differs from working execution"));
                }
            }
            return inspection.snapshot(self.storage()).await;
        }
        if execution.saved.is_some() {
            return Err(invalid("sealed cycle member has no saved execution"));
        }
        if let Some(snapshot) = self
            .execution_storage
            .retained_report_view(&execution.test_execution_id)
            .await
        {
            if snapshot.execution() != execution {
                return Err(invalid("retained snapshot differs from working execution"));
            }
            return Ok(snapshot);
        }
        let inputs = self
            .execution_storage
            .prepare_capture(cycle, execution, &self.storage().cursor_saved(), false)
            .await?;
        self.execution_storage
            .capture_execution_view(
                cycle,
                &inputs,
                self.settings().read(|document| document.report),
                &self.config().storage,
            )
            .await
    }
}

impl ExecutionStorage {
    pub(crate) async fn report_appearance(
        &self,
        view: &harmos::storage::StorageView,
        limit: NonZeroUsize,
    ) -> Result<QueryValue<crate::installed_plugins::RuntimeAppearanceFrame>> {
        let memory = self.storage.prepare_production(limit)?;
        let appearance = self
            .state_manager
            .state()
            .read(|state| {
                let brand = self.brand.read().expect("brand lock is healthy");
                crate::installed_plugins::appearance::capture_appearance_state(
                    state,
                    brand.as_ref(),
                    limit.get(),
                )
                .map_err(|_| invalid("report appearance exceeds its source budget"))
            })
            .await?
            .value?;
        let mut frame = appearance.frame;
        if let Some(asset) = appearance.logo {
            let url = super::presentation::capture_logo(&self.storage, view, &asset).await?;
            drop(asset);
            bounded(frame.memory_size().saturating_add(url.memory_size()), limit)?;
            if let Some(logo) = &mut frame.logo {
                logo.url = (*url).clone();
            }
        }
        Ok(memory.finish(frame)?.try_into_value()?)
    }
    pub(crate) async fn report_source(
        &self,
        execution: &TestExecutionId,
        settings: &crate::config::StorageSettings,
    ) -> Result<QueryValue<CycleView>> {
        let source = self
            .state_manager
            .state()
            .read(|state| {
                let id = &state
                    .executions
                    .get(execution)
                    .ok_or_else(|| invalid("report execution is not active"))?
                    .test_cycle_id;
                let header = state
                    .cycles
                    .get(id)
                    .ok_or_else(|| invalid("report cycle is not active"))?;
                let bytes = header.execution_ids.iter().try_fold(
                    header
                        .memory_size()
                        .saturating_add(std::mem::size_of::<CycleView>()),
                    |bytes, id| {
                        state
                            .executions
                            .get(id)
                            .and_then(|member| bytes.checked_add(member.memory_size()))
                            .ok_or_else(|| invalid("report cycle membership memory overflows"))
                    },
                )?;
                self.storage
                    .produce(bounded(bytes, settings.decoded_bytes)?, |_| {
                        state.cycle_view(id).ok_or(harmos::Error::InvalidInput {
                            field: "report cycle",
                            reason: "cycle disappeared",
                        })
                    })?
                    .try_into_value()
                    .map_err(Error::from)
            })
            .await?
            .value?;
        let lifecycle = self
            .state_manager
            .state()
            .read(|state| {
                state
                    .report_finalizations
                    .get(execution)
                    .and_then(|intent| intent.lifecycle.clone())
            })
            .await?
            .value;
        let Some(lifecycle) = lifecycle else {
            return Ok(source);
        };
        let bytes = source
            .memory_size()
            .saturating_add(lifecycle.execution.memory_size());
        Ok(
            source.try_project(bounded(bytes, settings.decoded_bytes)?, |source, _| {
                let mut source = source.clone();
                let member = source
                    .test_executions
                    .iter_mut()
                    .find(|member| member.test_execution_id == *execution)
                    .ok_or(harmos::Error::InvalidInput {
                        field: "report source",
                        reason: "execution is absent from cycle",
                    })?;
                *member = lifecycle.execution;
                Ok(source)
            })?,
        )
    }

    pub(crate) async fn capture_execution_view(
        &self,
        source: &CycleView,
        inputs: &CaptureInputs,
        defaults: crate::config::ReportDefaults,
        settings: &crate::config::StorageSettings,
    ) -> Result<ExecutionView> {
        let through = &*inputs.through;
        if !self.storage.cursor_saved().covers(through)? {
            return Err(invalid("report evidence is not durable"));
        }
        for output in inputs.outputs.iter() {
            if !through.covers(&output.file.position)? {
                return Err(invalid("report evidence precedes its captured output"));
            }
        }
        let view = self
            .storage
            .view_with_limits(settings.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let anchor = ExecutionView::open(
            &self.storage,
            view.clone(),
            &inputs.execution.test_execution_id,
            ObjectVersion::Latest,
            limit,
        )
        .await?;
        let revision = anchor
            .as_ref()
            .map(|anchor| {
                anchor
                    .revision()
                    .map(|revision| dokime::execution::saved::ExecutionRevision {
                        execution_id: revision.value.test_execution_id.clone(),
                        version: revision.version.clone(),
                        position: revision.position.clone(),
                    })
            })
            .transpose()?;
        let captured_at = dokime::core::Timestamp::now_utc();
        let capture_work = view.begin_computation()?;
        let preparation = self.storage.prepare_production(limit)?;
        let (mut context, logo, owner_node_id, record_contracts, telemetry_contracts) = self
            .state_manager
            .state()
            .read(|state| {
                let brand = self.brand.read().unwrap();
                let captured = super::presentation::capture_prepared(
                    state,
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
                        .saturating_add(revision.memory_size())
                        .saturating_add(inputs.execution.memory_size())
                        .saturating_add(inputs.lifecycle_id.memory_size())
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
                Ok::<_, Error>((
                    captured.context,
                    captured.logo,
                    state.identity.node.node_id.clone(),
                    records.clone(),
                    telemetry,
                ))
            })
            .await?
            .value?;
        context.health.push(crate::execution::HealthItem {
            health_item_id: dokime::core::HealthItemId::new(),
            scope: inputs.execution.scope.clone(),
            severity: crate::execution::HealthSeverity::Info,
            message: "Report Evidence Provenance".into(),
            detail: Some(format!(
                "Execution outcomes preserve recorded lifecycle state (capture {}). Chart samples use persisted evidence through storage cursor {}. Recorded verdicts may include outcomes whose raw samples were not retained at that boundary.",
                inputs.lifecycle_id, through.sequence,
            )),
            observed_at: captured_at.clone(),
        });
        for path in inputs.unavailable_outputs.iter() {
            context.health.push(crate::execution::HealthItem {
                health_item_id: dokime::core::HealthItemId::new(),
                scope: inputs.execution.scope.clone(),
                severity: crate::execution::HealthSeverity::Warning,
                message: format!("{path}: Unfinished At Stop — Not Included"),
                detail: Some("The file was not finished at the persisted report boundary.".into()),
                observed_at: captured_at.clone(),
            });
        }
        check(context.memory_size(), limit)?;
        // A plot nobody configured draws with the application defaults, frozen
        // here so a report rendered again from this capture draws the same.
        for document in context.execution_documents.values_mut() {
            defaults.apply(document);
        }
        if let Some(asset) = logo {
            let captured = super::presentation::capture_logo(&self.storage, &view, &asset).await?;
            if let Some(logo) = &mut context.appearance.logo {
                logo.url = (*captured).clone();
            }
        }
        let captured = preparation
            .finish(CapturedSource {
                captured_at,
                owner_node_id: owner_node_id.clone(),
                execution_id: inputs.execution.test_execution_id.clone(),
                execution: revision,
                lifecycle_id: inputs.lifecycle_id.clone(),
                lifecycle: (*inputs.execution).clone(),
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
            &self.storage,
            view.clone(),
            captured.clone(),
            &owner_node_id,
            limit,
            settings.finalization_read_stall,
        )
        .await?;
        Ok(inspection)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(|| invalid("report capture memory overflows"))
}
fn check(bytes: usize, allowance: NonZeroUsize) -> harmos::Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        return Err(harmos::Error::ResourceExhausted {
            resource: "report capture memory",
        });
    }
    Ok(())
}
fn bounded(bytes: usize, limit: NonZeroUsize) -> Result<NonZeroUsize> {
    let bytes = bound(bytes)?;
    if bytes > limit {
        return Err(exhausted().into());
    }
    Ok(bytes)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "cycle report capture",
    }
}
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

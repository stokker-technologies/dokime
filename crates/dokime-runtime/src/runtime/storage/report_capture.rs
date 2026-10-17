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
use crate::execution::context::captured::source::CapturedSource;
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use harmos::storage::ObjectVersion;

// Local Imports
use super::execution::ExecutionView;
use crate::{
    runtime::Error, runtime::Result, RuntimeHandle,
    execution::{CycleView, TestExecution},
};
use crate::core::TestExecutionId;

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
        execution: &crate::core::TestExecutionId,
    ) -> Result<ExecutionView> {
        let identity = execution.clone();
        let runtime = self.clone();
        tokio::spawn(async move {
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let source = runtime.snapshot_source(&identity).await?;
            let execution = source
                .test_executions
                .iter()
                .find(|member| member.test_execution_id == identity)
                .ok_or_else(|| invalid("snapshot execution is outside its cycle"))?;
            runtime.snapshot_member(&source, execution).await
        })
        .await?
    }

    async fn snapshot_source(&self, execution: &TestExecutionId) -> Result<CycleView> {
        self.execution_storage.report_source(execution).await
    }

    pub(super) async fn snapshot_member(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<ExecutionView> {
        if let Some(inspection) = ExecutionView::open(
            self.storage(),
            &execution.test_execution_id,
            ObjectVersion::Latest,
        )
        .await?
            && inspection.execution().saved.is_some()
        {
            if execution.saved.is_some() && inspection.execution() != execution {
                return Err(invalid("saved report member differs from frozen cycle"));
            }
            if execution.saved.is_none() {
                let mut expected = inspection.revision()?.value.clone();
                expected.saved = None;
                if &expected != execution {
                    return Err(invalid("saved snapshot differs from working execution"));
                }
            }
            return inspection.snapshot().await;
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
            )
            .await
    }
}

impl ExecutionStorage {
    pub(crate) async fn report_appearance(
        &self,
    ) -> Result<crate::installed_plugins::RuntimeAppearanceFrame> {
        let appearance = self
            .state_manager
            .state()
            .read(|state| {
                let brand = self.brand.read().expect("brand lock is healthy");
                crate::installed_plugins::appearance::capture_appearance_state(
                    state,
                    brand.as_ref(),
                    usize::MAX,
                )
                .map_err(|_| invalid("report appearance exceeds its source budget"))
            })
            .await?
            .value?;
        let mut frame = appearance.frame;
        if let Some(asset) = appearance.logo {
            let url = super::presentation::capture_logo(&asset).await?;
            if let Some(logo) = &mut frame.logo {
                logo.url = url;
            }
        }
        Ok(frame)
    }

    pub(crate) async fn report_source(&self, execution: &TestExecutionId) -> Result<CycleView> {
        let source = self
            .state_manager
            .state()
            .read(|state| {
                let id = &state
                    .executions
                    .get(execution)
                    .ok_or_else(|| invalid("report execution is not active"))?
                    .test_cycle_id;
                state
                    .cycle_view(id)
                    .ok_or_else(|| invalid("report cycle is not active"))
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
        let mut source = source;
        let member = source
            .test_executions
            .iter_mut()
            .find(|member| member.test_execution_id == *execution)
            .ok_or_else(|| invalid("execution is absent from cycle"))?;
        *member = self.frozen_execution(&lifecycle.execution).await?;
        Ok(source)
    }

    pub(crate) async fn capture_execution_view(
        &self,
        source: &CycleView,
        inputs: &CaptureInputs,
        defaults: crate::config::ReportDefaults,
    ) -> Result<ExecutionView> {
        let through = &inputs.through;
        if !self.storage.cursor_saved().covers(through)? {
            return Err(invalid("report evidence is not durable"));
        }
        let saved = self.storage.cursor_saved();
        for output in inputs.outputs.iter() {
            if !saved.covers(&output.file.position)? {
                return Err(invalid("captured report output is not durable"));
            }
        }
        let version = inputs
            .anchor
            .as_ref()
            .map_or(ObjectVersion::Latest, |anchor| {
                ObjectVersion::Exact(anchor.version.clone())
            });
        let anchor =
            ExecutionView::open(&self.storage, &inputs.execution.test_execution_id, version)
                .await?;
        let revision = anchor
            .as_ref()
            .map(|anchor| {
                anchor
                    .revision()
                    .map(|revision| crate::execution::saved::ExecutionRevision {
                        execution_id: revision.value.test_execution_id.clone(),
                        version: revision.version.clone(),
                        position: revision.position.clone(),
                    })
            })
            .transpose()?;
        let captured_at = crate::model::time::now_utc();
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
            health_item_id: crate::model::identity::new_id::<crate::core::HealthItemId>(),
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
                health_item_id: crate::model::identity::new_id::<crate::core::HealthItemId>(),
                scope: inputs.execution.scope.clone(),
                severity: crate::execution::HealthSeverity::Warning,
                message: format!("{path}: Unfinished At Stop — Not Included"),
                detail: Some("The file was not finished at the persisted report boundary.".into()),
                observed_at: captured_at.clone(),
            });
        }
        // A plot nobody configured draws with the application defaults, frozen
        // here so a report rendered again from this capture draws the same.
        for document in context.execution_documents.values_mut() {
            defaults.apply(document);
        }
        if let Some(asset) = logo {
            let captured = super::presentation::capture_logo(&asset).await?;
            if let Some(logo) = &mut context.appearance.logo {
                logo.url = captured;
            }
        }
        let captured = Arc::new(CapturedSource {
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
        });
        ExecutionView::captured(&self.storage, captured, &owner_node_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */

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
pub(crate) use crate::transactions::RecordHealth;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::execution::HealthItem;
use crate::{runtime::Error, runtime::Result as RuntimeResult, RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn record_health_item(&self, mut health: HealthItem) -> RuntimeResult<()> {
        let runtime = self.clone();
        tokio::spawn(async move {
            let _admission = runtime.evidence_admission.lock().await;
            health.scope = runtime.resolve_evidence_scope(&health.scope)?;
            let scope = health.scope.clone();
            let guards = runtime.execution_storage.admit_annotations(&scope).await?;
            runtime.record_health_item_inner(health).await?;
            drop(guards);
            runtime.checkpoint_evidence_scope(&scope).await
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
    }

    async fn record_health_item_inner(&self, health: HealthItem) -> RuntimeResult<()> {
        self.commit(RecordHealth {
            scope: health.scope,
            severity: health.severity,
            message: health.message,
            detail: health.detail,
            health_item_id: health.health_item_id,
            observed_at: health.observed_at,
        })
        .await
        .map_err(|error| Error::Operation(format!("failed to record runtime health: {error}")))?;
        Ok(())
    }

    pub(crate) fn resolve_evidence_scope(
        &self,
        scope: &dokime_domain::model::execution::RecordScope,
    ) -> RuntimeResult<dokime_domain::model::execution::RecordScope> {
        if (scope.step_id.is_some() || scope.criterion_id.is_some())
            && scope.test_execution_id.is_none()
        {
            return Err(Error::Operation(
                "step and criterion annotations require an execution owner".into(),
            ));
        }
        if scope.test_cycle_id.is_none()
            && scope.test_case_id.is_none()
            && scope.test_execution_id.is_none()
        {
            return Ok(scope.clone());
        }
        let state = self.state_read();
        let cycle = state
            .cycles
            .values()
            .find(|cycle| {
                scope
                    .test_cycle_id
                    .as_ref()
                    .is_none_or(|id| id == &cycle.test_cycle_id)
                    && scope.test_case_id.as_ref().is_none_or(|id| {
                        state
                            .cases_for_cycle(cycle)
                            .any(|case| &case.test_case_id == id)
                    })
                    && scope.test_execution_id.as_ref().is_none_or(|id| {
                        state.executions_for_cycle(cycle).any(|execution| {
                            &execution.test_execution_id == id
                                && scope
                                    .test_case_id
                                    .as_ref()
                                    .is_none_or(|case| case == &execution.test_case_id)
                        })
                    })
            })
            .ok_or_else(|| Error::Operation("annotation scope has no open runtime owner".into()))?;
        let mut scope = scope.clone();
        scope.test_cycle_id = Some(cycle.test_cycle_id.clone());
        Ok(scope)
    }

    pub(crate) async fn checkpoint_evidence_scope(
        &self,
        scope: &dokime_domain::model::execution::RecordScope,
    ) -> RuntimeResult<()> {
        let owners = {
            let snapshot = self.state_read();
            snapshot
                .cycles
                .keys()
                .filter(|id| {
                    scope
                        .test_cycle_id
                        .as_ref()
                        .is_none_or(|selected| selected == *id)
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let mut failure = None;
        for id in owners {
            // Save Each Member Once Before Publishing Its Grouping Header.
            if let Err(error) = self.execution_storage.save_cycle(id).await {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), |error| {
            Err(Error::Operation(format!(
                "annotation committed but checkpoint failed: {error}"
            )))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

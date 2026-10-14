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

// Third Party Imports

// Local Imports
use crate::{RuntimeHandle, execution::CycleStatus};
use dokime::core::{Identifier, RecordScope, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn finalize_execution_aggregate_outputs(
        &self,
        test_execution_id: &TestExecutionId,
    ) {
        let scope = self
            .state_read()
            .executions
            .get(test_execution_id)
            .map(|execution| execution.scope.clone());
        let Some(scope) = scope else {
            return;
        };
        let outputs = match self.telemetry_aggregate_outputs(&scope).await {
            Ok(outputs) => outputs,
            Err(error) => {
                tracing::warn!(%error, %test_execution_id, "failed to aggregate execution telemetry");
                return;
            }
        };
        let _ = self
            .commit(crate::transactions::outputs::FinalizeExecutionOutputs {
                test_execution_id: test_execution_id.clone(),
                outputs,
            })
            .await;
    }

    pub async fn update_execution_phase(
        &self,
        scope: RecordScope,
        identifier: Identifier,
        label: String,
        status: String,
    ) -> crate::Result<()> {
        self.commit_execution_phase(crate::traceability_runtime::phase::UpdateExecutionPhase {
            scope,
            identifier,
            label,
            status,
            occurred_at: Timestamp::now_utc(),
        })
        .await
    }

    pub async fn cycle_lifecycle(&self, test_cycle_id: &TestCycleId) -> Option<CycleStatus> {
        self.state_read()
            .cycles
            .get(test_cycle_id)
            .map(|cycle| cycle.lifecycle)
    }

    pub async fn wait_if_paused(&self, test_cycle_id: &TestCycleId) {
        let mut state = self.watch_state();
        state
            .wait_until(|snapshot| {
                snapshot.state().shutting_down
                    || !matches!(
                        snapshot
                            .state()
                            .cycles
                            .get(test_cycle_id)
                            .map(|cycle| cycle.lifecycle),
                        Some(CycleStatus::Paused),
                    )
            })
            .await;
    }

    pub(crate) async fn wait_until_cycle_stopped(&self, test_cycle_id: Option<&TestCycleId>) {
        let mut state = self.watch_state();
        state
            .wait_until(|snapshot| {
                snapshot.state().shutting_down
                    || test_cycle_id.is_some_and(|test_cycle_id| {
                        cycle_is_stopped(
                            snapshot
                                .state()
                                .cycles
                                .get(test_cycle_id)
                                .map(|cycle| cycle.lifecycle),
                        )
                    })
            })
            .await;
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn cycle_is_stopped(lifecycle: Option<CycleStatus>) -> bool {
    matches!(
        lifecycle,
        None | Some(
            CycleStatus::Stopping
                | CycleStatus::Stopped
                | CycleStatus::Completed
                | CycleStatus::Failed
                | CycleStatus::Blocked
                | CycleStatus::Cleared
        )
    )
}

/* === Tests === */

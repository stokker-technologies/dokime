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
pub(crate) use crate::transactions::{CycleResetAction as Action, ResetCycle as Reset};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::transactions::cycle::apply_cycle_reset;
use crate::{CommandContext, runtime::Error, RuntimeHandle};
use crate::core::{CaseId, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn delete_cycle_case(
        &self,
        test_cycle_id: TestCycleId,
        test_case_id: CaseId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.reset_cycle(test_cycle_id, Action::Delete { test_case_id }, context)
            .await
    }

    pub async fn rerun_cycle_case(
        &self,
        test_cycle_id: TestCycleId,
        test_case_id: CaseId,
        context: CommandContext,
    ) -> Result<(), Error> {
        self.reset_cycle(
            test_cycle_id,
            Action::Rerun {
                test_case_id,
                execution_id: TestExecutionId::new(),
            },
            context,
        )
        .await
    }

    async fn reset_cycle(
        &self,
        test_cycle_id: TestCycleId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        let runtime = self.clone();
        tokio::spawn(async move {
            runtime
                .reset_cycle_owned(test_cycle_id, action, context)
                .await
        })
        .await?
    }

    // Retain the admission through publication if the requesting UI disconnects.
    async fn reset_cycle_owned(
        &self,
        test_cycle_id: TestCycleId,
        action: Action,
        context: CommandContext,
    ) -> Result<(), Error> {
        let _admission = self.evidence_admission.lock().await;
        let transaction = Reset {
            test_cycle_id: test_cycle_id.clone(),
            action,
            occurred_at: Timestamp::now_utc(),
        };
        let mut projected = (*self.state_read()).clone();
        let execution_ids = apply_cycle_reset(&mut projected, &transaction)?;
        self.commit_with_key(context.idempotency_key, transaction)
            .await?;
        for execution_id in execution_ids {
            if let Err(error) = self.abandon_execution_artifacts(&execution_id).await {
                tracing::error!(%execution_id, %error, "failed to abandon execution artifacts");
            }
        }
        self.execution_storage
            .save_cycle(test_cycle_id.clone())
            .await?;
        self.reconcile_sampling().await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

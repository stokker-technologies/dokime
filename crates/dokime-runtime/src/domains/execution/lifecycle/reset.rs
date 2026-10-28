/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::transactions::{CycleResetAction as Action, ResetCycle as Reset};

use crate::transactions::cycle::plan_cycle_reset;
use crate::{CommandContext, RuntimeHandle, runtime::Error};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestCycleId;

impl RuntimeHandle {
    pub async fn delete_cycle_case(&self, test_cycle_id: TestCycleId, test_case_id: CaseId, context: CommandContext) -> Result<(), Error> {
        self.reset_cycle(test_cycle_id, Action::Delete { test_case_id }, context).await
    }

    async fn reset_cycle(&self, test_cycle_id: TestCycleId, action: Action, context: CommandContext) -> Result<(), Error> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.reset_cycle_owned(test_cycle_id, action, context).await }).await?
    }

    // Retain the admission through publication if the requesting UI disconnects.
    async fn reset_cycle_owned(&self, test_cycle_id: TestCycleId, action: Action, context: CommandContext) -> Result<(), Error> {
        let _admission = self.evidence_admission.lock().await;
        let transaction = Reset {
            test_cycle_id: test_cycle_id.clone(),
            action,
            occurred_at: crate::model::time::now_utc(),
        };
        let (_, execution_ids) = plan_cycle_reset(&self.state_read(), &transaction)?;
        self.commit_with_key(context.idempotency_key, transaction).await?;
        for execution_id in execution_ids {
            if let Err(error) = self.abandon_execution_artifacts(&execution_id).await {
                tracing::error!(%execution_id, %error, "failed to abandon execution artifacts");
            }
        }
        self.execution_storage.save_cycle(test_cycle_id.clone()).await?;
        self.reconcile_sampling().await;
        Ok(())
    }
}

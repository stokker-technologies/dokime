/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read And Answer Execution-Scoped Prompts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::TestExecutionHandle;
use crate::{Error, ResponseStream, Result, TestCycleHandle};
use dokime_domain::model::identity::{PromptId, TestCycleId};
use dokime_domain::model::prompts::wire::{PromptInstance, PromptResponseReceipt, PromptView};
use dokime_domain::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TestExecutionHandle {
    pub async fn pending_prompts(&self) -> Result<ResponseStream<PromptInstance>> {
        self.prompt_cycle().await?.pending_for_execution(Some(self.id.clone()))
    }

    pub async fn prompt(&self, id: &PromptId) -> Result<PromptView> {
        let view = self.prompt_cycle().await?.prompt(id).await?;
        self.validate_prompt(&view)?;
        Ok(view)
    }

    pub async fn respond_prompt(&self, id: &PromptId, action: String, values: Value, accepted: bool) -> Result<PromptResponseReceipt> {
        let cycle = self.prompt_cycle().await?;
        let view = cycle.prompt(id).await?;
        self.validate_prompt(&view)?;
        cycle.respond_prompt(id, action, values, accepted).await
    }

    async fn prompt_cycle(&self) -> Result<TestCycleHandle> {
        let detail = self.snapshot().await?;
        let cycle = TestCycleId::from_uuid(&detail.cycle_id)
            .map_err(|_| Error::Rpc(tonic::Status::data_loss("execution has an invalid cycle identity")))?;
        Ok(self.client.cycle(cycle))
    }

    fn validate_prompt(&self, view: &PromptView) -> Result<()> {
        let execution = view.prompt.as_ref().and_then(|prompt| prompt.scope.as_ref()).and_then(|scope| scope.execution_id.as_deref());
        if execution != Some(self.id.as_str()) {
            return Err(Error::Rpc(tonic::Status::invalid_argument("prompt belongs to another execution")));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

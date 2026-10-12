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
pub use crate::runtime::storage::requests::{AttemptClaim, ReceiptLoan, RunReceipt};
pub use crate::transactions::provider::request::{RunPhase, RunSummary};
pub use dokime::execution::request::{StepCommand, StepCommandPhase, StepCommandReceipt};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::{MemorySize, storage::QueryValue};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl crate::RuntimeHandle {
    pub async fn admit_execution_command(
        &self,
        request_id: &dokime::core::CommandId,
        command_id: dokime::core::CommandId,
        execution_id: dokime::core::TestExecutionId,
        action: dokime::execution::request::StepCommand,
    ) -> crate::Result<QueryValue<dokime::execution::request::StepCommandReceipt>> {
        let at = dokime::core::Timestamp::now_utc();
        let saved = self
            .run_request(request_id)
            .await?
            .ok_or_else(|| command_refused("run request not found"))?;
        let known = saved
            .known_step_command(&command_id, &execution_id, &action)
            .map_err(command_refused)?;
        if let Some(known) = known {
            let bytes = std::num::NonZeroUsize::new(known.memory_size()).ok_or_else(|| {
                crate::Error::Operation("empty command projection allowance".into())
            })?;
            return saved
                .try_project(bytes, |receipt, _allowance| {
                    Ok(receipt.commands[&command_id].clone())
                })
                .map_err(|error| crate::Error::RequestReceipt(Box::new(error)));
        }
        if self.is_shutting_down() {
            return Err(command_refused("runtime is shutting down"));
        }
        if let dokime::execution::request::StepCommand::RunStep { step_id } = &action {
            let state = self.state_read();
            let execution = state
                .executions
                .get(&execution_id)
                .ok_or_else(|| command_refused("execution not found"))?;
            if !execution.snapshot.template().is_some_and(|template| {
                template
                    .phases
                    .iter()
                    .any(|phase| &phase.step_id == step_id)
            }) {
                return Err(command_refused("step is absent from the frozen execution"));
            }
        }
        let growth = harmos::btree_map_memory::<
            dokime::core::CommandId,
            dokime::execution::request::StepCommandReceipt,
        >(1)
        .saturating_add(command_id.memory_size().saturating_mul(2))
        .saturating_add(execution_id.memory_size())
        .saturating_add(action.memory_size())
        .saturating_add(at.memory_size().saturating_mul(2));
        let selected = command_id.clone();
        let mut decision = None;
        self.request_storage
            .update(request_id, growth, |receipt| {
                decision = Some(
                    receipt
                        .admit_step_command(command_id, execution_id, action, at)
                        .map(|_| ()),
                );
            })
            .await?;
        decision
            .ok_or_else(|| command_refused("run request settled during command admission"))?
            .map_err(command_refused)?;
        self.request_storage
            .command(request_id, &selected)
            .await?
            .ok_or_else(|| crate::Error::Operation("admitted command receipt is absent".into()))
    }

    pub async fn run_request(
        &self,
        id: &dokime::core::CommandId,
    ) -> crate::Result<Option<ReceiptLoan>> {
        self.request_storage.get(id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn command_refused(message: impl Into<String>) -> crate::Error {
    crate::Error::Conflict {
        code: "execution_command_refused".into(),
        message: message.into(),
    }
}

/* === Tests === */

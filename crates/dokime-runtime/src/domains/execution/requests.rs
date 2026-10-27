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

pub use crate::runtime::storage::requests::{AttemptClaim, Receipt, RunReceipt, StepReceipt};
pub use crate::transactions::provider::request::{RunPhase, RunSummary};
pub use dokime_domain::model::execution::request::StepCommand;
pub use dokime_domain::model::execution::request::StepCommandPhase;
pub use dokime_domain::model::execution::request::StepCommandReceipt;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::diagnostics::admission::AdmissionFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct CommandWake<'a> {
    runtime: &'a crate::RuntimeHandle,
    execution: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for CommandWake<'_> {
    fn drop(&mut self) {
        self.runtime.execution_sessions.signal_change(&self.execution);
    }
}

impl crate::RuntimeHandle {
    pub async fn admit_execution_command(
        &self, request_id: &dokime_domain::model::identity::CommandId, command_id: dokime_domain::model::identity::CommandId,
        execution_id: dokime_domain::model::identity::TestExecutionId, action: dokime_domain::model::execution::request::StepCommand,
    ) -> crate::runtime::Result<StepReceipt> {
        let at = crate::model::time::now_utc();
        let saved = self.run_request(request_id).await?.ok_or_else(|| command_refused("run request not found"))?;
        let known = saved.known_step_command(&command_id, &execution_id, &action).map_err(command_refused)?;
        if known.is_some() {
            return saved.command(&command_id).ok_or_else(|| command_refused("known command receipt is absent"));
        }
        if self.is_shutting_down() {
            return Err(command_refused("runtime is shutting down"));
        }
        if let dokime_domain::model::execution::request::StepCommand::RunStep { step_id } = &action {
            let state = self.state_read();
            let execution = state.executions.get(&execution_id).ok_or_else(|| command_refused("execution not found"))?;
            if !execution.snapshot.template().is_some_and(|template| template.phases.iter().any(|phase| &phase.step_id == step_id)) {
                return Err(command_refused("step is absent from the frozen execution"));
            }
        }
        let selected = command_id.clone();
        // A Cancelled Caller May Leave A Durable Pending Receipt To Reconcile.
        let _changed = CommandWake { runtime: self, execution: execution_id.to_string() };
        let mut decision = None;

        self.request_storage
            .update(request_id, |receipt| {
                decision = Some(receipt.admit_step_command(command_id, execution_id, action, at).map(|_| ()));
            })
            .await?;
        decision.ok_or_else(|| command_refused("run request settled during command admission"))?.map_err(command_refused)?;
        self.request_storage
            .command(request_id, &selected)
            .await?
            .ok_or_else(|| crate::runtime::Error::Operation("admitted command receipt is absent".into()))
    }

    pub async fn run_request(&self, id: &dokime_domain::model::identity::CommandId) -> crate::runtime::Result<Option<Receipt>> {
        self.request_storage.get(id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn command_refused(message: impl Into<String>) -> crate::runtime::Error {
    crate::runtime::Error::Admission(AdmissionFailure::Conflict { code: "execution_command_refused".into(), message: message.into() })
}

/* === Tests === */

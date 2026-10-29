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
use super::{ExecutionFixtureContext, Target, TestExecution, failure};
use crate::transport::{CommandExecutionRequest, ProviderExecutionFailure};
use dokime_domain::model::execution::request::StepCommand;
use dokime_domain::model::execution::request::StepCommandPhase;
use dokime_domain::model::execution::request::StepCommandReceipt;
use dokime_domain::model::execution::termination::TerminationReason;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::StepAttemptId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(super) struct Pending {
    receipt: Option<StepCommandReceipt>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Pending {
    pub(super) fn invalidate(&mut self) {
        self.receipt = None;
    }

    async fn load<F>(&mut self, load: impl FnOnce() -> F) -> Result<Option<&mut StepCommandReceipt>, ProviderExecutionFailure>
    where
        F: std::future::Future<Output = Result<Option<StepCommandReceipt>, ProviderExecutionFailure>>,
    {
        if self.receipt.is_none() {
            self.receipt = load().await?;
        }
        Ok(self.receipt.as_mut())
    }
}

/* === Public Functions === */

// Answers Whether The Execution Still Has An Unsettled Command; Only Admission Reloads Its Durable Receipt.
pub(super) async fn reconcile(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, pending: &mut Pending, terminal: bool, cancelling: bool,
) -> Result<bool, ProviderExecutionFailure> {
    let Some(request_id) = &context.request_id else {
        return Ok(false);
    };
    let command = pending
        .load(|| async {
            let stored = context
                .runtime
                .request_storage
                .pending_command(request_id)
                .await
                .map_err(|error| failure("command_checkpoint", error.to_string()))?;
            Ok(stored.map(|receipt| receipt.as_ref().clone()))
        })
        .await?;
    let Some(command) = command else {
        return Ok(false);
    };
    if command.execution_id != execution.test_execution_id {
        return Err(failure("command_checkpoint", "command belongs to another execution"));
    }
    if command.phase == StepCommandPhase::Accepted {
        if terminal || cancelling {
            let detail = Some("Execution ended before command dispatch.".into());
            return save(context, request_id, command, StepCommandPhase::Interrupted, None, detail).await;
        }
        // The marker precedes the external call. Restart never resumes this intent.
        save(context, request_id, command, StepCommandPhase::Dispatching, None, None).await?;
    }
    let (action, step) = match &command.action {
        StepCommand::Next => ("next", String::new()),
        StepCommand::Finish => ("finish", String::new()),
        StepCommand::RunStep { step_id } => {
            let phase = execution
                .snapshot
                .template()
                .and_then(|template| template.phases.iter().find(|phase| &phase.step_id == step_id))
                .ok_or_else(|| failure("command_dispatch", "step is absent from the frozen execution"))?;
            ("run_step", phase.metadata.identifier.to_string())
        }
    };
    let reply = target
        .signal(crate::transport::ExecutionSignal {
            execution_token: execution.test_execution_id.to_string(),
            command: Some(CommandExecutionRequest {
                execution_token: execution.test_execution_id.to_string(),
                command_id: command.command_id.to_string(),
                action: action.into(),
                step,
            }),
            ..crate::transport::ExecutionSignal::default()
        })
        .await?
        .command
        .ok_or_else(|| failure("command_dispatch", "the run answered no command receipt"))?;
    let phase = match reply.state.as_str() {
        "accepted" => StepCommandPhase::Dispatching,
        "running" => StepCommandPhase::Running,
        "completed" => StepCommandPhase::Completed,
        "refused" => StepCommandPhase::Refused,
        "failed" => StepCommandPhase::Failed,
        _ => return Err(failure("command_dispatch", "unknown command receipt state")),
    };
    let attempt_id = (!reply.attempt_id.is_empty())
        .then(|| StepAttemptId::from_uuid(reply.attempt_id))
        .transpose()
        .map_err(|error| failure("command_dispatch", error.to_string()))?;
    if phase == StepCommandPhase::Completed || phase == StepCommandPhase::Failed {
        if command.action == StepCommand::Finish {
            if attempt_id.is_some() || phase == StepCommandPhase::Failed {
                return Err(failure("command_dispatch", "finish receipt has step provenance"));
            }
            if !terminal {
                return Ok(true);
            }
        } else {
            let id = attempt_id.as_ref().ok_or_else(|| failure("command_dispatch", "completed step command has no attempt receipt"))?;
            let state = context.runtime.state_read();
            let admitted = state
                .executions
                .get(&execution.test_execution_id)
                .and_then(|execution| execution.step_attempts.iter().find(|attempt| &attempt.attempt_id == id));
            let Some(admitted) = admitted else {
                if terminal {
                    return Err(failure("command_dispatch", "terminal command references an unadmitted attempt"));
                }
                return Ok(true);
            };
            if let StepCommand::RunStep { step_id } = &command.action
                && &admitted.step_id != step_id
            {
                return Err(failure("command_dispatch", "command receipt belongs to another step"));
            }
            if (phase == StepCommandPhase::Completed) != (admitted.termination == TerminationReason::Completed) {
                return Err(failure("command_dispatch", "command outcome differs from its admitted attempt"));
            }
        }
    } else if attempt_id.is_some() {
        return Err(failure("command_dispatch", "nonterminal or refused command has unexpected attempt provenance"));
    }
    save(context, request_id, command, phase, attempt_id, (!reply.message.is_empty()).then_some(reply.message)).await
}

/* === Private Functions === */

async fn save(
    context: &ExecutionFixtureContext, request_id: &CommandId, command: &mut StepCommandReceipt, phase: StepCommandPhase,
    attempt_id: Option<StepAttemptId>, detail: Option<String>,
) -> Result<bool, ProviderExecutionFailure> {
    let open = !phase.terminal();
    if command.phase == phase && command.attempt_id == attempt_id && command.detail == detail {
        return Ok(open);
    }
    let at = crate::model::time::now_utc();
    let cached_attempt = attempt_id.clone();
    let cached_detail = detail.clone();
    let cached_at = at.clone();
    let mut decision = None;

    context
        .runtime
        .request_storage
        .update(request_id, |receipt| {
            decision = Some(receipt.reconcile_step_command(&command.command_id, phase, attempt_id, detail, at));
        })
        .await
        .map_err(|error| failure("command_checkpoint", error.to_string()))?;
    decision
        .ok_or_else(|| failure("command_checkpoint", "run settled before command reconciliation"))?
        .map_err(|error| failure("command_checkpoint", error))?;
    command.phase = phase;
    command.attempt_id = cached_attempt;
    command.detail = cached_detail;
    command.updated_at = cached_at;
    Ok(open)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::identity::TestExecutionId;

    #[tokio::test]
    async fn long_command_polls_query_storage_only_after_own_admissions() {
        const EXECUTIONS: usize = 16;
        const COMMITS: usize = 10_000;
        const OWN_COMMANDS: usize = 3;
        let at = crate::model::time::now_utc();
        let receipt = StepCommandReceipt {
            command_id: CommandId::from_seed("cached-command"),
            execution_id: TestExecutionId::from_seed("cached-execution"),
            action: StepCommand::Next,
            phase: StepCommandPhase::Accepted,
            attempt_id: None,
            accepted_at: at.clone(),
            updated_at: at,
            detail: None,
        };
        let mut pending = (0..EXECUTIONS).map(|_| Pending::default()).collect::<Vec<_>>();
        let mut queries = [0; EXECUTIONS];
        for index in 0..EXECUTIONS {
            pending[index]
                .load(|| {
                    queries[index] += 1;
                    std::future::ready(Ok(Some(receipt.clone())))
                })
                .await
                .unwrap();
        }

        for commit in 0..COMMITS {
            for index in 0..EXECUTIONS {
                let spacing = COMMITS / OWN_COMMANDS;
                if commit % spacing == index && commit / spacing < OWN_COMMANDS {
                    pending[index].invalidate();
                }
                // Even Polling Every Global Commit Cannot Reload An Already Cached Receipt.
                pending[index]
                    .load(|| {
                        queries[index] += 1;
                        std::future::ready(Ok(Some(receipt.clone())))
                    })
                    .await
                    .unwrap();
            }
        }
        assert_eq!(queries, [1 + OWN_COMMANDS; EXECUTIONS]);
    }
}

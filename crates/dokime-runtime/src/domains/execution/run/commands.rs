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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn reconcile(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, terminal: bool, cancelling: bool,
) -> Result<(), ProviderExecutionFailure> {
    let Some(request_id) = &context.request_id else {
        return Ok(());
    };
    let Some(command) = context
        .runtime
        .request_storage
        .pending_command(request_id)
        .await
        .map_err(|error| failure("command_checkpoint", error.to_string()))?
    else {
        return Ok(());
    };
    if command.execution_id != execution.test_execution_id {
        return Err(failure("command_checkpoint", "command belongs to another execution"));
    }
    if command.phase == StepCommandPhase::Accepted {
        if terminal || cancelling {
            return save(
                context,
                request_id,
                &command,
                StepCommandPhase::Interrupted,
                None,
                Some("Execution ended before command dispatch.".into()),
            )
            .await;
        }
        // The marker precedes the external call. Restart never resumes this intent.
        save(context, request_id, &command, StepCommandPhase::Dispatching, None, None).await?;
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
                return Ok(());
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
                return Ok(());
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
    save(context, request_id, &command, phase, attempt_id, (!reply.message.is_empty()).then_some(reply.message)).await
}

/* === Private Functions === */
async fn save(
    context: &ExecutionFixtureContext, request_id: &CommandId, command: &StepCommandReceipt, phase: StepCommandPhase,
    attempt_id: Option<StepAttemptId>, detail: Option<String>,
) -> Result<(), ProviderExecutionFailure> {
    if command.phase == phase && command.attempt_id == attempt_id && command.detail == detail {
        return Ok(());
    }
    let at = crate::model::time::now_utc();
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
        .map(|_| ())
        .map_err(|error| failure("command_checkpoint", error))
}

/* === Tests === */

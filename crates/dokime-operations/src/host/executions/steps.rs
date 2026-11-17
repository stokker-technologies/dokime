/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Steps

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionStep;
use dokime_protocol::host::ExecutionsStepsRequest;
use dokime_protocol::host::ExecutionsStepsResponse;
use dokime_protocol::host::StepStatus;
use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Steps";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsStepsRequest) -> Result<ExecutionsStepsResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let position = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;

        let snapshot = crate::host::execution_state::evidence(context, &request.name)?;
        let case = crate::host::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?;
        let mut steps = Vec::new();
        for (index, step) in case.steps.iter().enumerate() {
            let identifier = step.metadata.as_ref().unwrap().identifier.clone();
            let evidence: Vec<_> = snapshot.evidence.iter().filter(|entry| entry.step == identifier).collect();
            let started_at =
                evidence.iter().find(|entry| entry.kind == "step:start").map(|entry| crate::host::projection::timestamp(entry.receipt_ns));
            let finished_at = evidence
                .iter()
                .rev()
                .find(|entry| entry.kind == "step:end")
                .map(|entry| crate::host::projection::timestamp(entry.receipt_ns));
            let status = if finished_at.is_some() {
                if evidence.iter().any(|entry| entry.kind == "step:failure") { StepStatus::Failed } else { StepStatus::Passed }
            } else if started_at.is_some() {
                if snapshot.state == ExecutionState::Sealed { StepStatus::Cancelled } else { StepStatus::Running }
            } else {
                StepStatus::Pending
            };
            steps.push(ExecutionStep {
                step_id: identifier,
                metadata: step.metadata.clone(),
                ordinal: index as u64,
                status: status as i32,
                started_at,
                finished_at,
                ..Default::default()
            });
        }
        Ok(ExecutionsStepsResponse { steps, resolved_at: Some(position) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsStepsRequest, ExecutionsStepsResponse, context, "dokime.host.v1", "Executions", "Steps");

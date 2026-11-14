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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Steps";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsStepsRequest, dokime_protocol::host::ExecutionsStepsResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsStepsRequest,
) -> Result<dokime_protocol::host::ExecutionsStepsResponse> {
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
                if evidence.iter().any(|entry| entry.kind == "step:failure") {
                    dokime_protocol::host::StepStatus::Failed
                } else {
                    dokime_protocol::host::StepStatus::Passed
                }
            } else if started_at.is_some() {
                if snapshot.state == dokime_storage::ExecutionState::Sealed {
                    dokime_protocol::host::StepStatus::Cancelled
                } else {
                    dokime_protocol::host::StepStatus::Running
                }
            } else {
                dokime_protocol::host::StepStatus::Pending
            };
            steps.push(dokime_protocol::host::ExecutionStep {
                step_id: identifier,
                metadata: step.metadata.clone(),
                ordinal: index as u64,
                status: status as i32,
                started_at,
                finished_at,
                ..Default::default()
            });
        }
        Ok(dokime_protocol::host::ExecutionsStepsResponse { steps, resolved_at: Some(position) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

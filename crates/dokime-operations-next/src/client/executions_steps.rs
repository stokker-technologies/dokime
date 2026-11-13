/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Executions/Steps";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsStepsRequest, v1::ExecutionsStepsResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsStepsRequest) -> Result<v1::ExecutionsStepsResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let position=context.host.store().resource_value(&request.name,"progress",request.at.as_ref())?.resolved_at;

        let snapshot = super::execution_state::evidence(context, &request.name)?;
        let case = super::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?;
        let mut steps = Vec::new();
        for (index, step) in case.steps.iter().enumerate() {
            let identifier = step.metadata.as_ref().unwrap().identifier.clone();
            let evidence: Vec<_> = snapshot.evidence.iter().filter(|entry| entry.step == identifier).collect();
            let started_at =
                evidence.iter().find(|entry| entry.kind == "step:start").map(|entry| super::projection::timestamp(entry.receipt_ns));
            let finished_at =
                evidence.iter().rev().find(|entry| entry.kind == "step:end").map(|entry| super::projection::timestamp(entry.receipt_ns));
            let status = if finished_at.is_some() {
                if evidence.iter().any(|entry| entry.kind == "step:failure") { v1::StepStatus::Failed } else { v1::StepStatus::Passed }
            } else if started_at.is_some() {
                if snapshot.state == dokime_storage::ExecutionState::Sealed { v1::StepStatus::Cancelled } else { v1::StepStatus::Running }
            } else {
                v1::StepStatus::Pending
            };
            steps.push(v1::ExecutionStep {
                step_id: identifier,
                metadata: step.metadata.clone(),
                ordinal: index as u64,
                status: status as i32,
                started_at,
                finished_at,
                ..Default::default()
            });
        }
        Ok(v1::ExecutionsStepsResponse { steps, resolved_at:Some(position) })
    }).await
}

/* === Private Functions === */

/* === Tests === */

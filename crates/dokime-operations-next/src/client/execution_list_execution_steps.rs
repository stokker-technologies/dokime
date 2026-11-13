/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Execution Steps

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_execution_steps";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListExecutionStepsRequest, v1::ListExecutionStepsReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionListExecutionStepsRequest) -> Result<v1::ListExecutionStepsReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let snapshot = super::execution_state::evidence(context, &request.execution_id)?;
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
        Ok(v1::ListExecutionStepsReply { steps })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

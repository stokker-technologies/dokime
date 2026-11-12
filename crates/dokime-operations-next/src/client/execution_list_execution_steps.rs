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
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
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
        let cursor =
            super::paging::cursor(context, format!("steps/{}", request.execution_id), snapshot.evidence.iter().map(Message::encode_to_vec));
        if request.cursor.as_ref().is_some_and(|supplied| {
            supplied.execution_id != request.execution_id
                || supplied.revision.as_ref() != Some(&cursor)
                || supplied.kind != v1::StepPageKind::Steps as i32
                || !supplied.step_id.is_empty()
        }) {
            return Err(Error::Precondition("List Steps: Restart Paging for the Selected Execution".into()));
        }
        let start = request.cursor.as_ref().map_or(0, |cursor| cursor.next_ordinal as usize);
        if start > case.steps.len() {
            return Err(Error::Invalid("List Steps: Restart Paging Within the Frozen Step List".into()));
        }
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let mut steps = Vec::new();
        for (index, step) in case.steps.iter().enumerate().skip(start).take(limit) {
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
        let next = start + steps.len();
        let next_cursor = (next < case.steps.len()).then(|| v1::StepPageCursor {
            revision: Some(cursor.clone()),
            execution_id: request.execution_id,
            kind: v1::StepPageKind::Steps as i32,
            next_ordinal: next as u64,
            ..Default::default()
        });
        Ok(v1::ListExecutionStepsReply {
            steps,
            next_cursor,
            observed_cursor: Some(cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

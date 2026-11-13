/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.get_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionGetExecutionRequest, v1::ExecutionDetail>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionGetExecutionRequest) -> Result<v1::ExecutionDetail> {
    super::complete(ID, context.settings.read_timeout, async {
        let snapshot = super::execution_state::read(context, &request.execution_id)?;
        let progress = context.host.store().resource_value(&request.execution_id, "progress", None)?;
        let controls = context.host.controls(&request.execution_id)?;
        let summary = super::execution_state::summary(context, &snapshot)?;
        let outcomes = context.host.store().cycle_outcomes(&snapshot.execution.cycle)?;
        let cycle_counts = super::projection::counts(&outcomes);
        let reports = context.host.store().reports(&request.execution_id)?;
        let schemas = super::projection::schemas(&snapshot)?;
        let (started_at, finished_at) = context.host.store().execution_times(&request.execution_id)?;
        Ok(v1::ExecutionDetail {
            progress: Some(v1::ExecutionProgressValue::decode(progress.payload.as_slice())?),
            resolved_at: Some(progress.resolved_at),
            started_at: started_at.map(super::projection::timestamp),
            finished_at: if snapshot.state == dokime_storage::ExecutionState::Sealed {
                finished_at.map(super::projection::timestamp)
            } else {
                None
            },
            input_parameter_count: snapshot
                .execution
                .origin
                .as_ref()
                .and_then(|origin| super::execution_state::case(origin).ok())
                .map_or(0, |case| case.parameters.len() as u64),
            step_count: snapshot
                .execution
                .origin
                .as_ref()
                .and_then(|origin| super::execution_state::case(origin).ok())
                .map_or(0, |case| case.steps.len() as u64),
            observed_cursor: (snapshot.state != dokime_storage::ExecutionState::Sealed).then(|| super::execution_state::observed(context)),
            summary: Some(summary),
            controls: Some(controls),
            cycle_counts: Some(cycle_counts),
            report: Some(v1::ReportStatus {
                state: if reports.is_empty() { v1::ReportState::Pending } else { v1::ReportState::Ready } as i32,
                ..Default::default()
            }),
            cycle_id: snapshot.execution.cycle.clone(),
            scope: Some(super::projection::scope(&snapshot)),
            origin: snapshot.execution.origin.clone(),
            outcome: snapshot.execution.outcome.clone(),
            seal: snapshot.seal.clone(),
            schemas: Some(schemas),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

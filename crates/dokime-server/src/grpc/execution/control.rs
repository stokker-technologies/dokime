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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Copy)]
pub(super) enum CaseOp {
    Delete,
    Rerun,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn observed(runtime: &RuntimeHandle) -> Response<v1::ControlReply> {
    Response::new(v1::ControlReply {
        observed_cursor: Some(observation::cursor(&runtime.watch_state().latest())),
    })
}

pub(super) async fn case_control(
    runtime: &RuntimeHandle,
    request: Request<v1::CaseControlRequest>,
    op: CaseOp,
) -> Result<Response<v1::ControlReply>, Status> {
    let context = context(&request, request.get_ref().context.as_ref())?;
    let (cycle, case) = case(request.get_ref())?;
    match op {
        CaseOp::Delete => runtime.delete_cycle_case(cycle, case, context).await,
        CaseOp::Rerun => runtime.rerun_cycle_case(cycle, case, context).await,
    }
    .map_err(status)?;
    Ok(observed(runtime))
}

pub(super) async fn clear_cycle(
    runtime: &RuntimeHandle,
    request: Request<v1::CycleControlRequest>,
) -> Result<Response<v1::ControlReply>, Status> {
    let context = context(&request, request.get_ref().context.as_ref())?;
    runtime
        .clear_cycle(cycle_id(&request.get_ref().cycle_id)?, context)
        .await
        .map_err(status)?;
    Ok(observed(runtime))
}

pub(super) async fn update_cycle_policy(
    runtime: &RuntimeHandle,
    request: Request<v1::UpdateCyclePolicyRequest>,
) -> Result<Response<v1::ControlReply>, Status> {
    let context = context(&request, request.get_ref().context.as_ref())?;
    runtime
        .update_cycle_policy(
            cycle_id(&request.get_ref().cycle_id)?,
            request.get_ref().continue_on_failure,
            context,
        )
        .await
        .map_err(status)?;
    Ok(observed(runtime))
}

/* === Private Functions === */
fn case(request: &v1::CaseControlRequest) -> Result<(TestCycleId, CaseId), Status> {
    value::text(&request.case_id)?;
    Ok((
        cycle_id(&request.cycle_id)?,
        CaseId::from_wire(request.case_id.clone())
            .map_err(|_| Status::invalid_argument("invalid case ID"))?,
    ))
}

/* === Tests === */

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
use super::codec::{self, RunWire};
use super::*;
use dokime_app::{
    capabilities::execution::requests::{RunPhase, RunReceipt},
    core::{CommandId, Timestamp},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn context<T>(
    request: &Request<T>,
    labels: Option<&v1::MutationContext>,
    key: Option<&str>,
) -> Result<CommandContext, Status> {
    let mut context = super::context(request, labels)?;
    if let Some(key) = key {
        dokime::core::command::validate_idempotency_key(key)
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        context.idempotency_key = Some(key.to_owned());
    }
    Ok(context)
}

pub(super) async fn get(
    runtime: &RuntimeHandle,
    request: Request<v1::GetRunRequestRequest>,
) -> Result<codec::RunReceipt, Status> {
    let principal = Principal::of(&request)?;
    let id = CommandId::from_uuid(request.get_ref().request_id.clone())
        .map_err(|_| Status::invalid_argument("invalid run request UUID"))?;
    let receipt = runtime
        .run_request(&id)
        .await
        .map_err(|error| {
            super::super::error::detailed(
                Status::unavailable("cannot resolve primary run receipt; retry lookup"),
                &error,
            )
        })?
        .ok_or_else(|| Status::not_found("run request not found"))?;
    authorize(principal, &receipt)?;
    let bytes = std::mem::size_of::<RunWire>()
        .saturating_add(receipt.summary.request_id.as_str().len())
        .saturating_add(receipt.node_id.as_str().len())
        .saturating_add(receipt.summary.cycle_id.as_str().len())
        .saturating_add(receipt.last_claim.as_ref().map_or(0, |claim| {
            claim
                .case_id
                .as_str()
                .len()
                .saturating_add(claim.execution_id.as_str().len())
        }))
        .saturating_add(receipt.submitted_by.as_ref().map_or(0, String::len))
        .saturating_add(receipt.interruption.as_ref().map_or(0, String::len));
    let preparation = codec::reserve(runtime, bytes)?;
    let output = RunWire(project(&receipt)?);
    preparation
        .finish(output)
        .map_err(codec::memory_error)?
        .try_into_value()
        .map_err(codec::memory_error)
}

pub(super) fn authorize(principal: &Principal, receipt: &RunReceipt) -> Result<(), Status> {
    if principal.role() != Role::Admin
        && receipt.submitted_by.as_deref() != Some(principal.subject())
    {
        return Err(Status::not_found("run request not found"));
    }
    Ok(())
}

/* === Private Functions === */
pub(super) fn timestamp(at: &Timestamp) -> Result<prost_types::Timestamp, Status> {
    let at =
        time::OffsetDateTime::parse(at.as_str(), &time::format_description::well_known::Rfc3339)
            .map_err(|_| Status::internal("invalid run receipt timestamp"))?;
    Ok(prost_types::Timestamp {
        seconds: at.unix_timestamp(),
        nanos: at.nanosecond() as i32,
    })
}

fn project(receipt: &RunReceipt) -> Result<v1::RunRequestReceipt, Status> {
    let summary = &receipt.summary;
    if let Some(interruption) = &receipt.interruption {
        value::text(interruption)?;
    }
    Ok(v1::RunRequestReceipt {
        request_id: summary.request_id.as_str().to_owned(),
        mode: match summary.mode {
            dokime_app::execution::ExecutionMode::Automatic => v1::ExecutionMode::Automatic as i32,
            dokime_app::execution::ExecutionMode::Interactive => {
                v1::ExecutionMode::Interactive as i32
            }
        },
        node_id: receipt.node_id.as_str().to_owned(),
        cycle_id: summary.cycle_id.as_str().to_owned(),
        phase: match summary.phase {
            RunPhase::Queued => v1::RunRequestPhase::Queued,
            RunPhase::Preparing => v1::RunRequestPhase::Preparing,
            RunPhase::Running => v1::RunRequestPhase::Running,
            RunPhase::Finishing => v1::RunRequestPhase::Finishing,
            RunPhase::Completed => v1::RunRequestPhase::Completed,
            RunPhase::Failed => v1::RunRequestPhase::Failed,
            RunPhase::Stopped => v1::RunRequestPhase::Stopped,
            RunPhase::Interrupted => v1::RunRequestPhase::Interrupted,
        } as i32,
        paused: summary.paused,
        stop_requested: summary.stop_requested,
        planned_count: summary.planned,
        prepared_count: summary.prepared,
        finished_count: summary.finished,
        unstarted_count: summary.unstarted,
        failed_count: summary.failed,
        claimed_count: receipt.claimed,
        last_claim: receipt
            .last_claim
            .as_ref()
            .map(|claim| v1::RunAttemptClaim {
                ordinal: claim.ordinal,
                case_id: claim.case_id.as_str().to_owned(),
                execution_id: claim.execution_id.as_str().to_owned(),
            }),
        accepted_at: Some(timestamp(&summary.accepted_at)?),
        provider_started_at: summary
            .provider_started_at
            .as_ref()
            .map(timestamp)
            .transpose()?,
        finished_at: summary.finished_at.as_ref().map(timestamp).transpose()?,
        submitted_by: receipt.submitted_by.clone(),
        interruption: receipt.interruption.clone(),
    })
}

/* === Tests === */

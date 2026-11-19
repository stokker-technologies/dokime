/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Seal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{CompoundOutcome, DeleteScope, DeleteScopeRequest};
use harmos_protocol::model::FileReference;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CyclePhase;
use dokime_protocol::host::CyclesSealRequest;
use dokime_protocol::host::CyclesSealResponse;
use dokime_protocol::host::{CycleSummary, EvidenceSubject, ReportSubject, ReportsCreateRequest, evidence_subject, report_subject};
use dokime_storage::ExecutionState;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Seal";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: CyclesSealRequest) -> Result<CyclesSealResponse> {
    let first = crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Seal Cycle: Supply a Request ID".into()));
        }
        let _guard = context.administration.lock().await;
        let mut cycle = crate::host::execution_state::cycle(context, &request.name).await?;
        if cycle.sealed || cycle.phase == CyclePhase::Sealing as i32 {
            return Ok(cycle);
        }
        for row in crate::host::execution_state::snapshots(context, &request.name)? {
            let execution = &row.execution.identifier;
            let controls = context.host.controls(execution)?;
            if controls.request_id.is_some()
                || controls.can_cancel
                || row.state != ExecutionState::Sealed && context.host.store().cut(execution)?.frontier != 0
            {
                return Err(Error::Precondition(format!("Seal Cycle {}: Wait for Running Execution {execution} to Seal", request.name)));
            }
        }
        // Retire the Previous Attempt Before Admitting Its Replacement.
        cleanup(context, &request.name).await?;
        cycle.phase = CyclePhase::Sealing as i32;
        cycle.seal_failure = None;
        cycle.updated_at = Some(crate::host::projection::timestamp(context.host.now_ns()));
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        let work = context.clone();
        let name = request.name.clone();
        if let Err(error) = context.host.spawn_work(Box::pin(async move {
            let result = materialize(&work, &name).await;
            let saved = async {
                let _guard = work.administration.lock().await;
                let mut cycle = crate::host::execution_state::cycle(&work, &name).await?;
                match result {
                    Ok((report, archive)) => {
                        cycle.report_file = Some(report);
                        cycle.evidence_file = Some(archive);
                        cycle.sealed = true;
                        cycle.phase = CyclePhase::Sealed as i32;
                        cycle.seal_failure = None;
                    }
                    Err(source) => {
                        cycle.sealed = false;
                        cycle.phase = CyclePhase::SealFailed as i32;
                        cycle.seal_failure = Some(format!("Seal Cycle {name}: {source}; Resolve the Cause and Retry Seal"));
                        cycle.updated_at = Some(crate::host::projection::timestamp(work.host.now_ns()));
                    }
                }
                // Commit Both Retained References With the Sealed State in One Write.
                // Successful Publication Retains the Seal Timestamp Frozen at Admission.
                work.host.store().save_cycle(&cycle)?;
                work.host.signal_change();
                Result::Ok(())
            }
            .await;
            if let Err(source) = saved {
                tracing::error!(cycle = %name, error = %source, "Save Cycle Seal Completion");
            }
        })) {
            cycle.phase = CyclePhase::SealFailed as i32;
            cycle.seal_failure = Some(error.to_string());
            context.host.store().save_cycle(&cycle)?;
            context.host.signal_change();
            return Err(error);
        }
        Ok(cycle)
    })
    .await?;
    response(context, first, &request.request_id)
}

pub async fn recover(context: &Context) -> Result<()> {
    for cycle in context.host.store().client_cycles()? {
        if !cycle.sealed {
            cleanup(context, &cycle.cycle_id).await?;
        }
    }
    Ok(())
}

/* === Private Functions === */

async fn cleanup(context: &Context, name: &str) -> Result<()> {
    let request_id = uuid::Uuid::now_v7().to_string();
    let mutation = crate::host::files::transfer::mutation(context, ID, &request_id).await?;
    let response = context
        .storage::<DeleteScope>(DeleteScopeRequest {
            mutation: Some(mutation),
            scope: Scope::CycleSeal(name).identifier()?,
        })
        .await?;
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict(format!("Retire Cycle {name} Seal Attempt: Reopen the Host Before Retrying Seal")));
    }
    Ok(())
}

fn response(context: &Context, cycle: CycleSummary, request: &str) -> Result<CyclesSealResponse> {
    let mut receipt = crate::host::views::receipt(context, request)?;
    receipt.completed = cycle.sealed || cycle.phase == CyclePhase::SealFailed as i32;
    Ok(CyclesSealResponse {
        seal: if cycle.sealed { crate::host::paging::bytes(&cycle) } else { Vec::new() },
        receipt: Some(receipt),
        cycle: Some(cycle),
    })
}

async fn materialize(context: &Context, name: &str) -> Result<(FileReference, FileReference)> {
    let scope = Scope::CycleSeal(name).identifier()?;
    {
        let _guard = context.administration.lock().await;
        for row in crate::host::execution_state::snapshots(context, name)? {
            if row.state != ExecutionState::Sealed {
                context
                    .host
                    .store()
                    .seal_unstarted(&row.execution.identifier, crate::host::projection::timestamp(context.host.now_ns()))?;
            }
        }
        context.host.signal_change();
    }
    let report = crate::host::reports::create::prepare(
        context,
        ReportsCreateRequest {
            subject: Some(ReportSubject {
                kind: Some(report_subject::Kind::Cycle(name.into())),
            }),
            request_id: format!("cycle-seal/{name}/{}", uuid::Uuid::now_v7()),
            ..Default::default()
        },
        Some(scope.clone()),
    )
    .await?;
    let file = report.file.ok_or_else(|| Error::Precondition("Seal Cycle: Restore Its Materialized Report".into()))?;
    let archive = crate::host::evidence::archive::materialize(
        context,
        EvidenceSubject {
            kind: Some(evidence_subject::Kind::Cycle(name.into())),
        },
        Some(file.clone()),
        Some(scope),
    )
    .await?;
    Ok((file, archive))
}

/* === Tests === */

crate::native::unary!(CyclesSealRequest, CyclesSealResponse, context, "dokime.host.v1", "Cycles", "Seal");

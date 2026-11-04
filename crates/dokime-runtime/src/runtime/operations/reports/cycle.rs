/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle Report Materialization

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use futures_util::{StreamExt, stream::FuturesOrdered};
use harmos::storage::StorageCursor;

// Local Imports
use super::Resources;
use crate::report::Scope;
use crate::{
    report::cycle::CycleMember,
    runtime::Error,
    runtime::Result,
    runtime::progress::Operation,
    runtime::storage::{cycle::CycleSnapshot, cycle_finalization},
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::saved::ExecutionSeal;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::reports::wire::MaterializedReport;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// How One Member Of An Active Cycle Enters Its Current Report.
enum Member {
    Planned(Box<TestExecution>),
    Sealed(harmos::storage::ReadBatch<ExecutionSeal>),
    // Its Materialization Gate Stays Held Until Its Evidence Is Materialized.
    Captured(crate::runtime::storage::execution::ExecutionView, tokio::sync::OwnedMutexGuard<()>),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn materialize(
    ctx: &Scope, resources: &Resources, id: &TestCycleId, at: &StorageCursor, progress: &Operation,
) -> Result<MaterializedReport> {
    let gate = resources.execution_storage.cycle_save.lock().await;
    progress.phase("Collecting Cycle Report Members");
    let finalization = resources.execution_storage.cycle_materialization(id, at, progress).await?;
    match finalization {
        // The Seal Keeps Saves Ordered Until Its One Report Is Retained.
        Some(finalization) => {
            let _gate = gate;
            sealed(ctx, resources, id, at, finalization, progress).await
        }
        None => current(ctx, resources, id, at, progress, gate).await,
    }
}

/* === Private Functions === */

// A Finalizing Cycle Renders From Its Frozen Snapshot; A Published Report Is Reused.
async fn sealed(
    ctx: &Scope, resources: &Resources, id: &TestCycleId, at: &StorageCursor, finalization: cycle_finalization::Materialization,
    progress: &Operation,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    if let Some(report) = finalization.published {
        return Ok(MaterializedReport { report });
    }
    let mut members = Vec::with_capacity(finalization.executions.len());
    let mut pending = FuturesOrdered::new();
    for execution in finalization.executions.iter() {
        if pending.len() == 4
            && let Some(member) = pending.next().await
        {
            members.push(member?);
        }
        pending.push_back(sealed_member(ctx, resources, execution, at, progress));
    }
    while let Some(member) = pending.next().await {
        members.push(member?);
    }
    drop(pending);
    progress.phase("Rendering");
    let prepared = crate::report::cycle::CycleReport::prepare(
        storage,
        finalization.snapshot,
        members,
        progress,
        &resources.settings.read_application_settings().await?.settings.report,
    )
    .await?;
    progress.phase("Publishing Report");
    let report = prepared.finish().await?;
    storage.flush_until(&report.position).await?;
    resources.execution_storage.retain_cycle_report(id, at, report.clone()).await?;
    Ok(MaterializedReport { report })
}

// An Active Cycle Renders Every Started Member At Its Own Persisted Boundary
// And Lists Every Unstarted Member As Planned, Keeping Cycle Order.
async fn current(
    ctx: &Scope, resources: &Resources, id: &TestCycleId, at: &StorageCursor, progress: &Operation, gate: tokio::sync::MutexGuard<'_, ()>,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    let state = ctx.state();
    let current = state
        .state()
        .read(|state| {
            let cycle = state.cycles.get(id).ok_or_else(|| invalid("cycle is not active"))?;
            // A Sealed Cycle Is Served As Published.
            if let Some(seal) = &cycle.saved {
                return Ok(Err(MaterializedReport { report: seal.report.clone() }));
            }
            let source = state.cycle_view(id).ok_or_else(|| invalid("cycle member is missing"))?;
            let boundaries = cycle
                .execution_ids
                .iter()
                .filter_map(|id| state.report_finalizations.get(id).map(|intent| (id.clone(), intent.evidence_at.clone())))
                .collect::<std::collections::BTreeMap<_, _>>();
            Ok::<_, Error>(Ok((source, state.identity.node.node_id.clone(), state.session_id.clone(), boundaries)))
        })
        .await?
        .value?;
    let (source, owner, session, intents) = match current {
        Ok(current) => current,
        Err(published) => return Ok(published),
    };
    let mut header = cycle_finalization::header(&source, None);
    if header.session_id.is_none() {
        header.session_id = Some(session);
    }
    // Members Are Admitted And Captured While Cycle Saves Wait; Their Evidence
    // Is Then Materialized Without Holding Notes Or Run Completions Back.
    let mut plans = Vec::new();
    for execution in source.test_executions.iter() {
        if unstarted(execution) {
            plans.push(Member::Planned(Box::new(execution.clone())));
            continue;
        }
        let seal = match &execution.saved {
            Some(seal) => Some(resources.execution_storage.retain_seal(seal)?),
            None => match resources.execution_storage.published_seal(&execution.test_execution_id).await? {
                Some(seal) => Some(seal),
                None => resources.execution_storage.durable_seal(&execution.test_execution_id).await?,
            },
        };
        let plan = if let Some(seal) = seal {
            Member::Sealed(seal)
        } else {
            progress.phase("Capturing Evidence");
            let boundary = intents.get(&execution.test_execution_id).unwrap_or(at);
            if !storage.cursor_saved().covers(boundary)? {
                return Err(invalid("pending member evidence is not durable"));
            }
            // A Finalizing Member Keeps Its Owner, Which Already Refuses New Producers.
            if resources.execution_storage.acquisition(&execution.test_execution_id).await.is_none() {
                resources.execution_storage.acquire(&source, execution).await?;
            }
            let guard = resources.execution_storage.materialization_gate(&execution.test_execution_id).await?.lock_owned().await;
            // Finalization May Publish This Member's Final Report While The Gate Was Awaited.
            if let Some(seal) = resources.execution_storage.published_seal(&execution.test_execution_id).await? {
                plans.push(Member::Sealed(seal));
                continue;
            }
            let inspection = match resources.execution_storage.published_report_view(&execution.test_execution_id).await? {
                Some(inspection) => inspection,
                None => {
                    let inputs = resources.execution_storage.prepare_capture(&source, execution, boundary, false).await?;
                    resources
                        .execution_storage
                        .capture_execution_view(&source, &inputs, resources.settings.read_application_settings().await?.settings.report)
                        .await?
                }
            };
            if inspection.evidence_through()? != boundary || inspection.execution() != execution {
                return Err(invalid("cycle member differs from its frozen source"));
            }
            Member::Captured(inspection, guard)
        };
        plans.push(plan);
    }
    let requests = plans
        .iter()
        .filter_map(|plan| match plan {
            Member::Captured(inspection, _) => Some(crate::report::materialize::evidence_request(inspection, false)),
            _ => None,
        })
        .collect::<Result<Vec<_>>>()?;
    // A Current Report Freezes Its Read After Member Admission. A Member Can
    // Seal While Admission Waits, Beyond The Caller's Earlier Durable Cursor.
    let at = &storage.cursor_saved();
    let mut evidence = ctx.storage.report_evidence(requests).await?.into_iter();
    drop(gate);
    let mut members = Vec::with_capacity(plans.len());
    let mut pending = FuturesOrdered::new();
    for plan in plans {
        if pending.len() == 4
            && let Some(member) = pending.next().await
        {
            members.push(member?);
        }
        let snapshot = if matches!(&plan, Member::Captured(_, _)) { evidence.next() } else { None };
        pending.push_back(async move {
            Ok::<_, Error>(match plan {
                Member::Planned(execution) => CycleMember::Planned(execution),
                Member::Sealed(seal) => sealed_member(ctx, resources, &seal, at, progress).await?,
                Member::Captured(inspection, _guard) => CycleMember::capture(
                    crate::report::materialize::execution_with_evidence(
                        ctx,
                        &inspection,
                        false,
                        progress,
                        snapshot.expect("captured member has evidence"),
                    )
                    .await?,
                )?,
            })
        });
    }
    while let Some(member) = pending.next().await {
        members.push(member?);
    }
    drop(pending);
    let captured = members.iter().find_map(|member| match member {
        CycleMember::Captured(captured) => Some(captured.read()),
        CycleMember::Planned(_) => None,
    });
    let appearance = match captured {
        Some(member) => member?.context.appearance.clone(),
        None => resources.execution_storage.report_appearance().await?,
    };
    let snapshot = CycleSnapshot::current(storage.cursor_saved(), Arc::new(header), Arc::new(appearance), &owner)?;
    progress.phase("Rendering");
    let prepared = crate::report::cycle::CycleReport::prepare(
        storage,
        snapshot,
        members,
        progress,
        &resources.settings.read_application_settings().await?.settings.report,
    )
    .await?;
    progress.phase("Publishing Report");
    let report = prepared.finish().await?;
    storage.flush_until(&report.position).await?;
    Ok(MaterializedReport { report })
}

// A Sealed Member Reuses The Input Its Final Report Materialized; Only A Member
// Sealed Before This Process Re-Queries Its Durable Context And Evidence.
async fn sealed_member(
    ctx: &Scope, resources: &Resources, seal: &ExecutionSeal, at: &StorageCursor, progress: &Operation,
) -> Result<CycleMember> {
    if !at.covers(&seal.source.evidence_through)? {
        return Err(invalid("sealed member evidence exceeds cycle report boundary"));
    }
    if let Some(member) = resources.execution_storage.sealed_cycle_member(seal).await {
        tracing::debug!(execution_id = %seal.source.execution.execution_id, "cycle report reuses the sealed member's spooled input");
        return Ok(CycleMember::Captured(member));
    }
    tracing::debug!(execution_id = %seal.source.execution.execution_id, "cycle report re-materializes a sealed member");
    let storage = &resources.execution_storage.storage;
    CycleMember::capture(crate::report::materialize::sealed_member(ctx, storage, seal, progress).await?)
}

// Nothing Dispatched It: Queued, Or Planned As The Runtime Spells It (An Unsealed Skip Never Started).
fn unstarted(execution: &TestExecution) -> bool {
    match execution.status {
        ExecutionStatus::Pending => true,
        ExecutionStatus::Skipped => execution.started_at.is_none() && execution.finished_at.is_none() && execution.saved.is_none(),
        _ => false,
    }
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

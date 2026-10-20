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
use harmos::{operations::Context, storage::StorageCursor};

// Local Imports
use super::Resources;
use crate::{
    app::Dokime,
    config::StorageSettings,
    report::CapturedExecutionReportInput,
    runtime::Error,
    runtime::Result,
    runtime::progress::Operation,
    runtime::storage::{cycle::CycleSnapshot, cycle_finalization},
    storage::Manager,
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::saved::ExecutionSeal;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::reports::wire::MaterializedReport;

/* === Constants === */

// Members This Process No Longer Retains Re-Query Their Evidence, At Most This Many At Once.
const ACTIVE_MEMBER_QUERIES: usize = 4;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn materialize(
    ctx: &Context<'_>, resources: &Resources, id: &TestCycleId, at: &StorageCursor, progress: &Operation,
) -> Result<MaterializedReport> {
    let _gate = resources.execution_storage.cycle_save.lock().await;
    progress.phase("Collecting Cycle Report Members");
    let finalization = resources.execution_storage.cycle_materialization(id, at, progress).await?;
    match finalization {
        Some(finalization) => sealed(ctx, resources, id, at, finalization, progress).await,
        None => current(ctx, resources, id, at, progress).await,
    }
}

/* === Private Functions === */

// A Finalizing Cycle Renders From Its Frozen Snapshot; A Published Report Is Reused.
async fn sealed(
    ctx: &Context<'_>, resources: &Resources, id: &TestCycleId, at: &StorageCursor, finalization: cycle_finalization::Materialization,
    progress: &Operation,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    let settings = &ctx.scope().resources::<Dokime>().storage_settings;
    if let Some(report) = finalization.published {
        return Ok(MaterializedReport { report });
    }
    let mut members = Vec::with_capacity(finalization.executions.len());
    let mut pending = FuturesOrdered::new();
    for execution in finalization.executions.iter() {
        if pending.len() == ACTIVE_MEMBER_QUERIES
            && let Some(member) = pending.next().await
        {
            members.push(member?);
        }
        pending.push_back(sealed_member(ctx, resources, storage, settings, execution, at, progress));
    }
    while let Some(member) = pending.next().await {
        members.push(member?);
    }
    drop(pending);
    progress.phase("Rendering");
    let prepared = crate::report::cycle::CycleReport::prepare(storage, finalization.snapshot, members, progress).await?;
    progress.phase("Publishing Report");
    let report = prepared.finish(ctx).await?;
    storage.flush_until(&report.position).await?;
    resources.execution_storage.retain_cycle_report(id, at, report.clone()).await?;
    Ok(MaterializedReport { report })
}

// An Active Cycle Renders Every Started Member At Its Own Persisted Boundary.
async fn current(
    ctx: &Context<'_>, resources: &Resources, id: &TestCycleId, at: &StorageCursor, progress: &Operation,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    let settings = &ctx.scope().resources::<Dokime>().storage_settings;
    let state = ctx.scope().state::<Dokime>();
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
    let mut members = Vec::new();
    for execution in source.test_executions.iter().filter(|execution| execution.status != ExecutionStatus::Pending) {
        let seal = match &execution.saved {
            Some(seal) => Some(seal.clone()),
            None => match resources.execution_storage.published_seal(&execution.test_execution_id).await {
                Some(seal) => Some(seal),
                None => resources.execution_storage.durable_seal(&execution.test_execution_id).await?,
            },
        };
        let member = if let Some(seal) = &seal {
            sealed_member(ctx, resources, storage, settings, seal, at, progress).await?
        } else {
            progress.phase("Capturing Evidence");
            let boundary = intents.get(&execution.test_execution_id).unwrap_or(at);
            if !at.covers(boundary)? {
                return Err(invalid("pending member evidence exceeds cycle report boundary"));
            }
            // A Finalizing Member Keeps Its Owner, Which Already Refuses New Producers.
            if resources.execution_storage.acquisition(&execution.test_execution_id).await.is_none() {
                resources.execution_storage.acquire(&source, execution).await?;
            }
            let gate = resources.execution_storage.materialization_gate(&execution.test_execution_id).await?;
            let _guard = gate.lock().await;
            // Finalization May Publish This Member's Final Report While The Gate Was Awaited.
            if let Some(seal) = resources.execution_storage.published_seal(&execution.test_execution_id).await {
                members.push(sealed_member(ctx, resources, storage, settings, &seal, at, progress).await?);
                continue;
            }
            let inspection = match resources.execution_storage.retained_report_view(&execution.test_execution_id).await {
                Some(inspection) => inspection,
                None => {
                    let inputs = resources.execution_storage.prepare_capture(&source, execution, boundary, false).await?;
                    resources
                        .execution_storage
                        .capture_execution_view(&source, &inputs, resources.settings.read(|settings| settings.report))
                        .await?
                }
            };
            if inspection.evidence_through()? != boundary || inspection.execution() != execution {
                return Err(invalid("cycle member differs from its frozen source"));
            }
            crate::report::materialize::execution(ctx, settings, &inspection, false, progress).await?
        };
        members.push(member);
    }
    let appearance = match members.first() {
        Some(member) => member.context.appearance.clone(),
        None => resources.execution_storage.report_appearance().await?,
    };
    let snapshot = CycleSnapshot::current(storage.cursor_saved(), Arc::new(header), Arc::new(appearance), &owner)?;
    progress.phase("Rendering");
    let prepared = crate::report::cycle::CycleReport::prepare(storage, snapshot, members, progress).await?;
    progress.phase("Publishing Report");
    let report = prepared.finish(ctx).await?;
    storage.flush_until(&report.position).await?;
    Ok(MaterializedReport { report })
}

// A Member Sealed By This Process Reuses Its Materialized Report Input; Others Re-Query Their Evidence.
async fn sealed_member(
    ctx: &Context<'_>, resources: &Resources, storage: &Manager, settings: &StorageSettings, seal: &ExecutionSeal, at: &StorageCursor,
    progress: &Operation,
) -> Result<CapturedExecutionReportInput> {
    if !at.covers(&seal.source.evidence_through)? {
        return Err(invalid("sealed member evidence exceeds cycle report boundary"));
    }
    if let Some(input) = resources.execution_storage.retained_member_input(seal).await {
        return Ok((*input).clone());
    }
    crate::report::materialize::sealed_member(ctx, storage, settings, seal, progress).await
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

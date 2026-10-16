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
use harmos::{operations::Context, storage::StorageCursor};

// Local Imports
use super::Resources;
use crate::{
    Error, Result,
    app::Dokime,
    runtime::storage::{cycle::CycleSnapshot, cycle_finalization},
};
use dokime::{
    core::TestCycleId, execution::ExecutionStatus, operations::reports::MaterializedReport,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn materialize(
    ctx: &Context<'_>,
    resources: &Resources,
    id: &TestCycleId,
    at: &StorageCursor,
) -> Result<MaterializedReport> {
    let _gate = resources.execution_storage.cycle_save.lock().await;
    let collecting = resources.progress.begin("Collecting Cycle Report Members");
    let finalization = resources
        .execution_storage
        .cycle_materialization(id, at, &collecting)
        .await?;
    collecting.finish();
    match finalization {
        Some(finalization) => sealed(ctx, resources, id, at, finalization).await,
        None => current(ctx, resources, id, at).await,
    }
}

/* === Private Functions === */
// A Finalizing Cycle Renders From Its Frozen Snapshot; A Published Report Is Reused.
async fn sealed(
    ctx: &Context<'_>,
    resources: &Resources,
    id: &TestCycleId,
    at: &StorageCursor,
    finalization: cycle_finalization::Materialization,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    let settings = &ctx.scope().resources::<Dokime>().storage_settings;
    if let Some(report) = finalization.published {
        return Ok(MaterializedReport { report });
    }
    let mut members = Vec::with_capacity(finalization.executions.len());
    for execution in finalization.executions.iter() {
        if !at.covers(&execution.source.evidence_through)? {
            return Err(invalid(
                "sealed member evidence exceeds cycle report boundary",
            ));
        }
        members.push(
            crate::report::materialize::sealed_member(ctx, storage, settings, execution).await?,
        );
    }
    let report =
        crate::report::cycle::CycleReport::prepare(storage, finalization.snapshot, members, None)
            .await?
            .finish(ctx)
            .await?;
    storage.flush_until(&report.position).await?;
    resources
        .execution_storage
        .retain_cycle_report(id, at, report.clone())
        .await?;
    Ok(MaterializedReport { report })
}

// An Active Cycle Renders Every Started Member At Its Own Persisted Boundary.
async fn current(
    ctx: &Context<'_>,
    resources: &Resources,
    id: &TestCycleId,
    at: &StorageCursor,
) -> Result<MaterializedReport> {
    let storage = &resources.execution_storage.storage;
    let settings = &ctx.scope().resources::<Dokime>().storage_settings;
    let state = ctx.scope().state::<Dokime>();
    let (source, owner, session, intents) = state
        .state()
        .read(|state| {
            let cycle = state
                .cycles
                .get(id)
                .ok_or_else(|| invalid("cycle is not active"))?;
            if cycle.saved.is_some() {
                return Err(invalid("sealed cycle must read its published report"));
            }
            let source = state
                .cycle_view(id)
                .ok_or_else(|| invalid("cycle member is missing"))?;
            let boundaries = cycle
                .execution_ids
                .iter()
                .filter_map(|id| {
                    state
                        .report_finalizations
                        .get(id)
                        .map(|intent| (id.clone(), intent.evidence_at.clone()))
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            Ok::<_, Error>((
                source,
                state.identity.node.node_id.clone(),
                state.session_id.clone(),
                boundaries,
            ))
        })
        .await?
        .value?;
    let mut header = cycle_finalization::header(&source, None);
    if header.session_id.is_none() {
        header.session_id = Some(session);
    }
    let mut members = Vec::new();
    for execution in source
        .test_executions
        .iter()
        .filter(|execution| execution.status != ExecutionStatus::Pending)
    {
        let member = if let Some(seal) = &execution.saved {
            if !at.covers(&seal.source.evidence_through)? {
                return Err(invalid(
                    "sealed member evidence exceeds cycle report boundary",
                ));
            }
            crate::report::materialize::sealed_member(ctx, storage, settings, seal).await?
        } else {
            let boundary = intents.get(&execution.test_execution_id).unwrap_or(at);
            if !at.covers(boundary)? {
                return Err(invalid(
                    "pending member evidence exceeds cycle report boundary",
                ));
            }
            resources
                .execution_storage
                .acquire(&source, execution)
                .await?;
            let gate = resources
                .execution_storage
                .materialization_gate(&execution.test_execution_id)
                .await?;
            let _guard = gate.lock().await;
            let inspection = match resources
                .execution_storage
                .retained_report_view(&execution.test_execution_id)
                .await
            {
                Some(inspection) => inspection,
                None => {
                    let inputs = resources
                        .execution_storage
                        .prepare_capture(&source, execution, boundary, false)
                        .await?;
                    resources
                        .execution_storage
                        .capture_execution_view(
                            &source,
                            &inputs,
                            resources.settings.read(|settings| settings.report),
                        )
                        .await?
                }
            };
            if inspection.evidence_through()? != boundary || inspection.execution() != execution {
                return Err(invalid("cycle member differs from its frozen source"));
            }
            crate::report::materialize::execution(ctx, settings, &inspection, false, None).await?
        };
        members.push(member);
    }
    let appearance = match members.first() {
        Some(member) => member.context.appearance.clone(),
        None => resources.execution_storage.report_appearance().await?,
    };
    let snapshot = CycleSnapshot::current(
        storage.cursor_saved(),
        Arc::new(header),
        Arc::new(appearance),
        &owner,
    )?;
    let report = crate::report::cycle::CycleReport::prepare(storage, snapshot, members, None)
        .await?
        .finish(ctx)
        .await?;
    storage.flush_until(&report.position).await?;
    Ok(MaterializedReport { report })
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

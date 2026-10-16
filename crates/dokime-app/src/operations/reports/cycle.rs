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

// Third Party Imports

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
use harmos::{MemorySize, operations::Context, storage::StorageCursor};
use std::num::NonZeroUsize;

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
    let storage = &resources.execution_storage.storage;
    let settings = &ctx.scope().resources::<Dokime>().storage_settings;
    let _gate = resources.execution_storage.cycle_save.lock().await;
    let collecting = resources.progress.begin("Collecting Cycle Report Members");
    if let Some((snapshot, executions, published, prepared)) = resources
        .execution_storage
        .cycle_materialization(id, at, settings.finalization(), &collecting)
        .await?
    {
        collecting.finish();
        if let Some(report) = published {
            return Ok(MaterializedReport {
                report: (*report).clone(),
            });
        }
        let mut prepared = prepared.lock().await;
        if prepared.is_none() {
            let _memory = storage.prepare_production(bound(executions.len().saturating_mul(
                std::mem::size_of::<
                    harmos::storage::QueryValue<crate::report::CapturedExecutionReportInput>,
                >(),
            ))?)?;
            let mut members = Vec::with_capacity(executions.len());
            for execution in executions.iter() {
                if !at.covers(&execution.source.evidence_through)? {
                    return Err(invalid(
                        "sealed member evidence exceeds cycle report boundary",
                    ));
                }
                members.push(
                    crate::report::materialize::sealed_member(
                        ctx,
                        storage,
                        settings,
                        snapshot.view(),
                        execution,
                    )
                    .await?,
                );
            }
            *prepared = Some(
                crate::report::cycle::CycleReport::prepare(storage, snapshot, members, None)
                    .await?,
            );
        }
        let report = prepared
            .as_mut()
            .expect("retained cycle artifact")
            .finish(ctx)
            .await?;
        storage.flush_until(&report.position).await?;
        resources
            .execution_storage
            .retain_cycle_report(id, at, report.clone())
            .await?;
        prepared.take();
        return Ok(MaterializedReport { report });
    }
    collecting.finish();
    let state = ctx.scope().state::<Dokime>();
    let source = state
        .state()
        .read(|state| {
            let cycle = state
                .cycles
                .get(id)
                .ok_or_else(|| invalid("cycle is not active"))?;
            if cycle.saved.is_some() {
                return Err(invalid("sealed cycle must read its published report"));
            }
            let bytes = cycle
                .execution_ids
                .iter()
                .try_fold(cycle.memory_size(), |bytes, id| {
                    let execution = state
                        .executions
                        .get(id)
                        .ok_or_else(|| invalid("cycle member is missing"))?;
                    bytes
                        .checked_add(execution.memory_size())
                        .ok_or_else(|| invalid("cycle source memory overflows"))
                })?
                .saturating_add(std::mem::size_of::<crate::execution::CycleView>());
            let source = storage
                .produce(bound(bytes)?, |_| {
                    state.cycle_view(id).ok_or(harmos::Error::InvalidInput {
                        field: "report cycle",
                        reason: "cycle disappeared",
                    })
                })?
                .try_into_value()?;
            let intent_bytes = cycle
                .execution_ids
                .iter()
                .filter_map(|id| {
                    state
                        .report_finalizations
                        .get(id)
                        .map(|intent| (id, &intent.evidence_at))
                })
                .fold(
                    harmos::btree_map_memory::<dokime::core::TestExecutionId, StorageCursor>(
                        cycle.execution_ids.len(),
                    ),
                    |bytes, (id, cursor)| {
                        bytes
                            .saturating_add(id.memory_size())
                            .saturating_add(cursor.memory_size())
                    },
                );
            let memory = storage.prepare_production(bound(
                intent_bytes
                    .saturating_add(state.identity.node.node_id.memory_size())
                    .saturating_add(state.session_id.memory_size()),
            )?)?;
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
                memory,
            ))
        })
        .await?
        .value?;
    let (source, owner, session, intents, _state_memory) = source;
    let header = storage
        .produce(
            bound(
                cycle_finalization::header_memory(&source).saturating_add(session.memory_size()),
            )?,
            |_| {
                let mut header = cycle_finalization::header(&source, None);
                if header.session_id.is_none() {
                    header.session_id = Some(session.clone());
                }
                Ok(header)
            },
        )?
        .try_into_value()?
        .try_into_shared()?;
    let count = source
        .test_executions
        .iter()
        .filter(|execution| execution.status != ExecutionStatus::Pending)
        .count();
    let _members_memory =
        storage.prepare_production(bound(count.saturating_mul(std::mem::size_of::<
            harmos::storage::QueryValue<crate::report::CapturedExecutionReportInput>,
        >()))?)?;
    let mut members = Vec::with_capacity(count);
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
            let view = storage.view_with_limits(settings.finalization()).await?;
            crate::report::materialize::sealed_member(ctx, storage, settings, &view, seal).await?
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
                            settings,
                        )
                        .await?
                }
            };
            if inspection.evidence_through()? != boundary || inspection.execution() != execution {
                return Err(invalid("cycle member differs from its frozen source"));
            }
            crate::report::materialize::execution(ctx, storage, settings, &inspection, false, None)
                .await?
        };
        members.push(member);
    }
    let view = storage.view_with_limits(settings.finalization()).await?;
    let appearance = if let Some(member) = members.first() {
        storage
            .produce(bound(member.context.appearance.memory_size())?, |_| {
                Ok(member.context.appearance.clone())
            })?
            .try_into_value()?
            .try_into_shared()?
    } else {
        resources
            .execution_storage
            .report_appearance(&view, view.decoded_bytes()?)
            .await?
            .try_into_shared()?
    };
    let snapshot = CycleSnapshot::current(storage, view, header, appearance, &owner)?;
    let mut report =
        crate::report::cycle::CycleReport::prepare(storage, snapshot, members, None).await?;
    let report = report.finish(ctx).await?;
    storage.flush_until(&report.position).await?;
    Ok(MaterializedReport { report })
}

/* === Private Functions === */
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes.max(1))
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| invalid("cycle report memory overflows"))
}
fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution State

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use dokime_storage::{ExecutionSnapshot, ExecutionState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn read(context: &Context, id: &str) -> Result<ExecutionSnapshot> {
    if !context.host.store().identifiers()?.iter().any(|identifier| identifier == id) {
        return Err(Error::NotFound("Inspect Execution: Select an Existing Execution".into()));
    }
    let cut = context.host.store().cut(id)?;
    Ok(ExecutionSnapshot {
        execution: cut.execution,
        state: cut.state,
        seal: cut.seal,
        evidence: Vec::new(),
        samples: Vec::new(),
    })
}
pub(super) fn evidence(context: &Context, id: &str) -> Result<ExecutionSnapshot> {
    let mut snapshot = read(context, id)?;
    let cut = context.host.store().cut(id)?;
    let limit = std::num::NonZeroUsize::new(context.settings.page_items).unwrap();
    let mut after = 0;
    loop {
        let page = context.host.store().evidence_page(id, after, cut.frontier, limit)?;
        if page.is_empty() {
            break;
        }
        after = page.last().unwrap().position;
        snapshot.evidence.extend(page);
    }
    Ok(snapshot)
}
pub(super) fn observed(context: &Context) -> v1::Cursor {
    v1::Cursor {
        epoch: context.host.identity().epoch,
        stream: "runtime.state".into(),
        position: *context.host.changes().borrow(),
    }
}
pub(super) fn case(origin: &v1::Origin) -> Result<&v1::TestCase> {
    match origin.kind.as_ref() {
        Some(v1::origin::Kind::PluginRun(run)) => {
            run.test_case.as_ref().ok_or_else(|| Error::Invalid("Inspect Case: Supply Its Frozen TestCase".into()))
        }
        _ => Err(Error::Invalid("Inspect Case: Select a PluginRun Origin".into())),
    }
}
pub(super) fn snapshots(context: &Context, cycle: &str) -> Result<Vec<ExecutionSnapshot>> {
    let mut rows = Vec::new();
    for id in context.host.store().identifiers()? {
        let snapshot = read(context, &id)?;
        if snapshot.execution.cycle == cycle {
            rows.push(snapshot);
        }
    }
    Ok(rows)
}
pub(super) fn summary(context: &Context, snapshot: &ExecutionSnapshot) -> Result<v1::ExecutionSummary> {
    let (case_id, _, archived) = context.host.store().execution_presentation(&snapshot.execution.identifier)?;
    let mut summary = super::projection::summary(snapshot, context.host.controls(&snapshot.execution.identifier)?);
    summary.case_id = case_id;
    summary.archived = archived;
    Ok(summary)
}
pub(super) fn cycle(context: &Context, id: &str) -> Result<v1::CycleSummary> {
    let rows = snapshots(context, id)?;
    let mut cycle = context
        .host
        .store()
        .client_cycle(id)?
        .or_else(|| {
            (!rows.is_empty()).then(|| v1::CycleSummary {
                cycle_id: id.into(),
                ..Default::default()
            })
        })
        .ok_or_else(|| Error::NotFound("Inspect Cycle: Select an Existing Cycle".into()))?;
    let mut running = 0;
    let mut queued = 0;
    let mut paused = cycle.status == v1::CycleStatus::Paused as i32;
    for row in &rows {
        let controls = context.host.controls(&row.execution.identifier)?;
        running += u64::from(controls.can_cancel);
        queued += u64::from(controls.can_start);
        paused |= controls.paused;
    }
    let mut cases: std::collections::BTreeSet<_> = context.host.store().planned_cases(id)?.into_iter().map(|case| case.case_id).collect();
    for row in rows
        .iter()
        .filter(|row| matches!(row.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(v1::origin::Kind::PluginRun(_))))
    {
        let (case, _, _) = context.host.store().execution_presentation(&row.execution.identifier)?;
        cases.insert(if case.is_empty() { row.execution.identifier.clone() } else { case });
    }
    let counts = super::projection::counts(rows.iter().filter_map(|row| row.execution.outcome.as_ref()));
    let mut result = v1::ResultSummary {
        status: counts.result_status.unwrap_or(v1::ResultStatus::Pending as i32),
        declared: counts.declared_count as u32,
        not_evaluated: counts.not_evaluated_count as u32,
        ..Default::default()
    };
    for row in &rows {
        if let Some(v1::execution_outcome::Attribution::Evaluated(outcome)) =
            row.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref())
        {
            match v1::ResultStatus::try_from(outcome.status).unwrap_or_default() {
                v1::ResultStatus::Passed => result.passed += 1,
                v1::ResultStatus::Failed => result.failed += 1,
                v1::ResultStatus::Blocked => result.blocked += 1,
                _ => {}
            }
        }
    }
    cycle.case_count = cases.len() as u64;
    cycle.execution_count = rows.len() as u64;
    cycle.running_count = running;
    cycle.queued_count = queued;
    cycle.phase = if cycle.sealed { v1::CyclePhase::Sealed } else { v1::CyclePhase::Active } as i32;
    cycle.status = if paused {
        v1::CycleStatus::Paused
    } else if running > 0 {
        v1::CycleStatus::Running
    } else if queued > 0 || rows.is_empty() {
        v1::CycleStatus::Ready
    } else if result.failed > 0 {
        v1::CycleStatus::Failed
    } else {
        v1::CycleStatus::Completed
    } as i32;
    cycle.result = Some(result);
    cycle.observed_cursor = Some(observed(context));
    Ok(cycle)
}
pub(super) fn cycles(context: &Context) -> Result<Vec<v1::CycleSummary>> {
    let mut ids: std::collections::BTreeSet<_> = context.host.store().client_cycles()?.into_iter().map(|cycle| cycle.cycle_id).collect();
    for id in context.host.store().identifiers()? {
        ids.insert(context.host.store().cut(&id)?.execution.cycle);
    }
    ids.into_iter().map(|id| cycle(context, &id)).collect()
}
pub(super) fn cases(context: &Context, cycle_id: &str) -> Result<Vec<v1::CaseSummary>> {
    cycle(context, cycle_id)?;
    let mut cases: BTreeMap<_, _> = context
        .host
        .store()
        .planned_cases(cycle_id)?
        .into_iter()
        .map(|case| {
            (
                case.case_id.clone(),
                v1::CaseSummary {
                    case_id: case.case_id,
                    template_id: case.template_id,
                    suite_id: case.suite_id,
                    metadata: case.metadata,
                    ordinal: case.ordinal,
                    status: v1::CaseStatus::Planned as i32,
                    actions: Some(v1::CaseActions {
                        delete: true,
                        unavailable_reasons: Vec::new(),
                    }),
                    ..Default::default()
                },
            )
        })
        .collect();
    for snapshot in snapshots(context, cycle_id)? {
        if !matches!(snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(v1::origin::Kind::PluginRun(_))) {
            continue;
        }
        let summary = summary(context, &snapshot)?;
        let id = if summary.case_id.is_empty() { snapshot.execution.identifier.clone() } else { summary.case_id };
        let run = snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref());
        let entry = cases.entry(id.clone()).or_insert_with(|| v1::CaseSummary {
            case_id: id,
            template_id: match run {
                Some(v1::origin::Kind::PluginRun(run)) => run.template.clone(),
                _ => String::new(),
            },
            ..Default::default()
        });
        entry.attempt_count += 1;
        entry.latest_execution_id = Some(snapshot.execution.identifier.clone());
        entry.latest_execution_status = Some(summary.state);
        entry.latest_execution_outcome = snapshot.execution.outcome.clone();
        let controls = context.host.controls(&snapshot.execution.identifier)?;
        entry.status = if controls.can_cancel {
            v1::CaseStatus::Running
        } else if snapshot.state != ExecutionState::Sealed {
            v1::CaseStatus::Queued
        } else {
            match snapshot.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()) {
                Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == v1::ResultStatus::Passed as i32 => {
                    v1::CaseStatus::Passed
                }
                Some(v1::execution_outcome::Attribution::Evaluated(_)) => v1::CaseStatus::Failed,
                Some(v1::execution_outcome::Attribution::NotEvaluated(outcome))
                    if outcome.termination == v1::Termination::Cancelled as i32 =>
                {
                    v1::CaseStatus::Cancelled
                }
                _ => v1::CaseStatus::Interrupted,
            }
        } as i32;
        entry.actions = Some(v1::CaseActions {
            delete: false,
            unavailable_reasons: vec!["The Case Has Execution Evidence".into()],
        });
    }
    let mut cases: Vec<_> = cases.into_values().collect();
    cases.sort_by(|left, right| left.ordinal.cmp(&right.ordinal).then_with(|| left.case_id.cmp(&right.case_id)));
    Ok(cases)
}
pub(super) fn planned(context: &Context, cycle: &str, case: &str) -> Result<v1::PlannedCase> {
    if !cases(context, cycle)?.iter().any(|entry| entry.case_id == case) {
        return Err(Error::NotFound("Inspect Case: Select a Case in This Cycle".into()));
    }
    if let Ok(case) = context.host.store().planned_case(case) {
        return Ok(case);
    }
    let snapshot = snapshots(context, cycle)?
        .into_iter()
        .find(|snapshot| snapshot.execution.identifier == case)
        .ok_or_else(|| Error::NotFound("Inspect Case: Select an Existing Frozen Case".into()))?;
    let template_id = match snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(v1::origin::Kind::PluginRun(run)) => run.template.clone(),
        _ => return Err(Error::NotFound("Inspect Case: Imports Have No Runnable Case".into())),
    };
    Ok(v1::PlannedCase {
        case_id: case.into(),
        template_id,
        origin: snapshot.execution.origin,
        ..Default::default()
    })
}
pub(super) fn parameters(
    context: &Context, parameters: &std::collections::HashMap<String, v1::Value>, limit: u32, after: &str,
    supplied: Option<&v1::ParameterCursor>, selection: v1::ParameterCursorSelection,
) -> Result<(Vec<v1::CaseParameter>, Option<String>, v1::ParameterCursor)> {
    let mut rows: Vec<_> = parameters
        .iter()
        .map(|(name, value)| v1::CaseParameter {
            name: name.clone(),
            value: Some(value.clone()),
        })
        .collect();
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    let current = super::paging::cursor(context, format!("parameters/{selection:?}"), rows.iter().map(super::paging::bytes));
    if supplied.is_some_and(|cursor| cursor.selection.as_ref() != Some(&selection)) {
        return Err(Error::Conflict("Read Parameters: Restart Paging with the Same Selection".into()));
    }
    let (rows, next) = super::paging::page(
        rows,
        limit,
        after,
        supplied.and_then(|cursor| cursor.observed.as_ref()),
        &current,
        context.settings.page_items,
        |row| &row.name,
    )?;
    Ok((
        rows,
        next,
        v1::ParameterCursor {
            observed: Some(current),
            selection: Some(selection),
        },
    ))
}
pub(super) fn archive_matches(scope: Option<i32>, archived: bool) -> Result<bool> {
    Ok(
        match scope
            .map(v1::ArchiveScope::try_from)
            .transpose()
            .map_err(|_| Error::Invalid("List: Choose a Valid Archive Scope".into()))?
            .unwrap_or(v1::ArchiveScope::Unarchived)
        {
            v1::ArchiveScope::Unarchived | v1::ArchiveScope::Unspecified => !archived,
            v1::ArchiveScope::Archived => archived,
            v1::ArchiveScope::All => true,
        },
    )
}
pub(super) fn time(value: Option<prost_types::Timestamp>) -> (i64, i32) {
    value.map(|value| (value.seconds, value.nanos)).unwrap_or_default()
}

/* === Private Functions === */

/* === Tests === */

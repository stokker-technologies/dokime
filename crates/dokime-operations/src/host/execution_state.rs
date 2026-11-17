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
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::{ReadCut, ResourceRevision};
use prost::Message;
use prost_types::Timestamp;

// Local Imports
use crate::host::{Context, cases, projection};
use crate::{Error, Result};
use dokime_protocol::host::ArchiveScope;
use dokime_protocol::host::CaseActions;
use dokime_protocol::host::CaseParameter;
use dokime_protocol::host::CaseStatus;
use dokime_protocol::host::CaseSummary;
use dokime_protocol::host::CyclePhase;
use dokime_protocol::host::CycleStatus;
use dokime_protocol::host::CycleSummary;
use dokime_protocol::host::ExecutionDetail;
use dokime_protocol::host::ExecutionSummary;
use dokime_protocol::host::PlannedCase;
use dokime_protocol::host::ReportState;
use dokime_protocol::host::ReportStatus;
use dokime_protocol::host::ResultStatus;
use dokime_protocol::host::ResultSummary;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::model::ExecutionProgressValue;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Termination;
use dokime_protocol::model::TestCase;
use dokime_protocol::model::Value;
use dokime_protocol::model::origin::Kind;
use dokime_storage::{ExecutionCut, ExecutionSnapshot, ExecutionState, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn read(context: &Context, id: &str) -> Result<ExecutionSnapshot> {
    let cut = existing_cut(context, id)?;
    Ok(ExecutionSnapshot {
        execution: cut.execution,
        state: cut.state,
        seal: cut.seal,
        evidence: Vec::new(),
        samples: Vec::new(),
    })
}
pub(crate) fn evidence(context: &Context, id: &str) -> Result<ExecutionSnapshot> {
    let store = context.host.store();
    let cut = existing_cut(context, id)?;
    evidence_at(&store, &cut, NonZeroUsize::new(context.settings.page_items).unwrap())
}
pub(crate) fn evidence_at(store: &Store, cut: &ExecutionCut, limit: NonZeroUsize) -> Result<ExecutionSnapshot> {
    let mut snapshot = ExecutionSnapshot {
        execution: cut.execution.clone(),
        state: cut.state,
        seal: cut.seal.clone(),
        evidence: Vec::new(),
        samples: Vec::new(),
    };
    let mut after = 0;
    loop {
        let page = store.evidence_page(&cut.execution.identifier, after, cut.frontier, limit)?;
        if page.is_empty() {
            break;
        }
        after = page.last().unwrap().position;
        snapshot.evidence.extend(page);
    }
    Ok(snapshot)
}
pub(crate) fn observed(context: &Context) -> ReadCut {
    ReadCut {
        positions: Vec::new(),
        revisions: vec![ResourceRevision {
            name: format!("runtime/{}/{}", context.host.identity().backend_id, context.host.identity().epoch),
            revision: context.host.changes().borrow().to_be_bytes().to_vec(),
        }],
    }
}
pub(crate) fn case(origin: &Origin) -> Result<&TestCase> {
    match origin.kind.as_ref() {
        Some(Kind::PluginRun(run)) => {
            run.test_case.as_ref().ok_or_else(|| Error::Invalid("Inspect Case: Supply Its Frozen TestCase".into()))
        }
        _ => Err(Error::Invalid("Inspect Case: Select a PluginRun Origin".into())),
    }
}
pub(crate) fn snapshots(context: &Context, cycle: &str) -> Result<Vec<ExecutionSnapshot>> {
    let mut rows = Vec::new();
    for id in context.host.store().identifiers()? {
        let snapshot = read(context, &id)?;
        if snapshot.execution.cycle == cycle {
            rows.push(snapshot);
        }
    }
    Ok(rows)
}
pub(crate) fn summary(context: &Context, snapshot: &ExecutionSnapshot) -> Result<ExecutionSummary> {
    let (case_id, _, archived) = context.host.store().execution_presentation(&snapshot.execution.identifier)?;
    let mut summary = projection::summary(snapshot, context.host.controls(&snapshot.execution.identifier)?);
    summary.case_id = case_id;
    summary.archived = archived;
    Ok(summary)
}
pub(crate) async fn cycle(context: &Context, id: &str) -> Result<CycleSummary> {
    let rows = snapshots(context, id)?;
    let mut cycle = context
        .host
        .store()
        .client_cycle(id)?
        .or_else(|| {
            (!rows.is_empty()).then(|| CycleSummary {
                cycle_id: id.into(),
                ..Default::default()
            })
        })
        .ok_or_else(|| Error::NotFound("Inspect Cycle: Select an Existing Cycle".into()))?;
    let mut running = 0;
    let mut queued = 0;
    let mut paused = cycle.status == CycleStatus::Paused as i32;
    for row in &rows {
        let controls = context.host.controls(&row.execution.identifier)?;
        running += u64::from(controls.can_cancel);
        queued += u64::from(controls.can_start);
        paused |= controls.paused;
    }
    let mut cases: BTreeSet<_> = cases::storage::list(context, id).await?.into_iter().map(|case| case.case_id).collect();
    for row in
        rows.iter().filter(|row| matches!(row.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(Kind::PluginRun(_))))
    {
        let (case, _, _) = context.host.store().execution_presentation(&row.execution.identifier)?;
        cases.insert(if case.is_empty() { row.execution.identifier.clone() } else { case });
    }
    let counts = projection::counts(rows.iter().filter_map(|row| row.execution.outcome.as_ref()));
    let mut result = ResultSummary {
        status: counts.result_status.unwrap_or(ResultStatus::Pending as i32),
        declared: counts.declared_count as u32,
        not_evaluated: counts.not_evaluated_count as u32,
        ..Default::default()
    };
    for row in &rows {
        if let Some(Attribution::Evaluated(outcome)) = row.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()) {
            match ResultStatus::try_from(outcome.status).unwrap_or_default() {
                ResultStatus::Passed => result.passed += 1,
                ResultStatus::Failed => result.failed += 1,
                ResultStatus::Blocked => result.blocked += 1,
                _ => {}
            }
        }
    }
    cycle.case_count = cases.len() as u64;
    cycle.execution_count = rows.len() as u64;
    cycle.running_count = running;
    cycle.queued_count = queued;
    cycle.phase = if cycle.sealed { CyclePhase::Sealed } else { CyclePhase::Active } as i32;
    cycle.status = if paused {
        CycleStatus::Paused
    } else if running > 0 {
        CycleStatus::Running
    } else if queued > 0 || rows.is_empty() {
        CycleStatus::Ready
    } else if result.failed > 0 {
        CycleStatus::Failed
    } else {
        CycleStatus::Completed
    } as i32;
    cycle.result = Some(result);
    cycle.observed_cursor = Some(observed(context));
    Ok(cycle)
}
pub(crate) async fn cycles(context: &Context) -> Result<Vec<CycleSummary>> {
    let mut ids: BTreeSet<_> = context.host.store().client_cycles()?.into_iter().map(|cycle| cycle.cycle_id).collect();
    for id in context.host.store().identifiers()? {
        ids.insert(context.host.store().cut(&id)?.execution.cycle);
    }
    let mut cycles = Vec::new();
    for id in ids {
        cycles.push(cycle(context, &id).await?);
    }
    Ok(cycles)
}
pub(crate) async fn cases(context: &Context, cycle_id: &str) -> Result<Vec<CaseSummary>> {
    cycle(context, cycle_id).await?;
    let mut cases: BTreeMap<_, _> = cases::storage::list(context, cycle_id)
        .await?
        .into_iter()
        .map(|case| {
            (
                case.case_id.clone(),
                CaseSummary {
                    case_id: case.case_id,
                    template_id: case.template_id,
                    suite_id: case.suite_id,
                    metadata: case.metadata,
                    ordinal: case.ordinal,
                    status: CaseStatus::Planned as i32,
                    actions: Some(CaseActions {
                        delete: true,
                        unavailable_reasons: Vec::new(),
                    }),
                    ..Default::default()
                },
            )
        })
        .collect();
    for snapshot in snapshots(context, cycle_id)? {
        if !matches!(snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(Kind::PluginRun(_))) {
            continue;
        }
        let summary = summary(context, &snapshot)?;
        let id = if summary.case_id.is_empty() { snapshot.execution.identifier.clone() } else { summary.case_id };
        let run = snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref());
        let entry = cases.entry(id.clone()).or_insert_with(|| CaseSummary {
            case_id: id,
            template_id: match run {
                Some(Kind::PluginRun(run)) => run.template.clone(),
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
            CaseStatus::Running
        } else if snapshot.state != ExecutionState::Sealed {
            CaseStatus::Queued
        } else {
            match snapshot.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()) {
                Some(Attribution::Evaluated(outcome)) if outcome.status == ResultStatus::Passed as i32 => CaseStatus::Passed,
                Some(Attribution::Evaluated(_)) => CaseStatus::Failed,
                Some(Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::Cancelled as i32 => CaseStatus::Cancelled,
                _ => CaseStatus::Interrupted,
            }
        } as i32;
        entry.actions = Some(CaseActions {
            delete: false,
            unavailable_reasons: vec!["The Case Has Execution Evidence".into()],
        });
    }
    let mut cases: Vec<_> = cases.into_values().collect();
    cases.sort_by(|left, right| left.ordinal.cmp(&right.ordinal).then_with(|| left.case_id.cmp(&right.case_id)));
    Ok(cases)
}
pub(crate) async fn planned(context: &Context, cycle: &str, case: &str) -> Result<PlannedCase> {
    if !cases(context, cycle).await?.iter().any(|entry| entry.case_id == case) {
        return Err(Error::NotFound("Inspect Case: Select a Case in This Cycle".into()));
    }
    match cases::storage::read(context, case, Some(cycle)).await {
        Ok(case) => return Ok(case),
        Err(Error::NotFound(_)) => {}
        Err(error) => return Err(error),
    }
    let snapshot = snapshots(context, cycle)?
        .into_iter()
        .find(|snapshot| snapshot.execution.identifier == case)
        .ok_or_else(|| Error::NotFound("Inspect Case: Select an Existing Frozen Case".into()))?;
    let template_id = match snapshot.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(Kind::PluginRun(run)) => run.template.clone(),
        _ => return Err(Error::NotFound("Inspect Case: Imports Have No Runnable Case".into())),
    };
    Ok(PlannedCase {
        case_id: case.into(),
        template_id,
        origin: snapshot.execution.origin,
        ..Default::default()
    })
}
pub(crate) fn parameters(parameters: &HashMap<String, Value>) -> Vec<CaseParameter> {
    let mut rows: Vec<_> = parameters
        .iter()
        .map(|(name, value)| CaseParameter {
            name: name.clone(),
            value: Some(value.clone()),
        })
        .collect();
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    rows
}
pub(crate) fn archive_matches(scope: Option<i32>, archived: bool) -> Result<bool> {
    Ok(
        match scope
            .map(ArchiveScope::try_from)
            .transpose()
            .map_err(|_| Error::Invalid("List: Choose a Valid Archive Scope".into()))?
            .unwrap_or(ArchiveScope::Unarchived)
        {
            ArchiveScope::Unarchived | ArchiveScope::Unspecified => !archived,
            ArchiveScope::Archived => archived,
            ArchiveScope::All => true,
        },
    )
}
pub(crate) fn time(value: Option<Timestamp>) -> (i64, i32) {
    value.map(|value| (value.seconds, value.nanos)).unwrap_or_default()
}

pub(crate) fn detail(context: &Context, name: &str) -> Result<ExecutionDetail> {
    let snapshot = read(context, name)?;
    let progress = context.host.store().resource_value(name, "progress", None)?;
    let controls = context.host.controls(name)?;
    let summary = summary(context, &snapshot)?;
    let outcomes = context.host.store().cycle_outcomes(&snapshot.execution.cycle)?;
    let cycle_counts = projection::counts(&outcomes);
    let reports = context.host.store().reports(name)?;
    let schemas = projection::schemas(&snapshot)?;
    let (started_at, finished_at) = context.host.store().execution_times(name)?;
    Ok(ExecutionDetail {
        progress: Some(ExecutionProgressValue::decode(progress.payload.as_slice())?),
        resolved_at: Some(progress.resolved_at),
        started_at: started_at.map(projection::timestamp),
        finished_at: if snapshot.state == ExecutionState::Sealed { finished_at.map(projection::timestamp) } else { None },
        input_parameter_count: snapshot
            .execution
            .origin
            .as_ref()
            .and_then(|origin| case(origin).ok())
            .map_or(0, |case| case.parameters.len() as u64),
        step_count: snapshot.execution.origin.as_ref().and_then(|origin| case(origin).ok()).map_or(0, |case| case.steps.len() as u64),
        summary: Some(summary),
        controls: Some(controls),
        cycle_counts: Some(cycle_counts),
        report: Some(ReportStatus {
            state: if reports.is_empty() { ReportState::Pending } else { ReportState::Ready } as i32,
            ..Default::default()
        }),
        cycle_id: snapshot.execution.cycle.clone(),
        scope: Some(projection::scope(&snapshot)),
        origin: snapshot.execution.origin.clone(),
        outcome: snapshot.execution.outcome.clone(),
        seal: snapshot.seal.clone(),
        schemas: Some(schemas),
        ..Default::default()
    })
}

/* === Private Functions === */

fn existing_cut(context: &Context, id: &str) -> Result<ExecutionCut> {
    let store = context.host.store();
    if !store.identifiers()?.iter().any(|identifier| identifier == id) {
        return Err(Error::NotFound("Inspect Execution: Select an Existing Execution".into()));
    }
    Ok(store.cut(id)?)
}

/* === Tests === */

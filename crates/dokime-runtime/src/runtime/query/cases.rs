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

pub use dokime_domain::model::execution::result::TestCaseStatus as CaseStatus;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::{MemorySize, storage::ObjectVersion};
use serde::{Deserialize, Serialize};

// Local Imports
use super::RuntimeStateSnapshot;
use crate::state::RuntimeState;
use crate::transactions::cycle::ensure_case_deletable;
use crate::{
    RuntimeHandle,
    runtime::{Error, Result as RuntimeResult},
};
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

const HISTORICAL: &str = "historical checkpoints cannot be modified";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CaseInspection {
    pub snapshot: RuntimeStateSnapshot,
    pub case: harmos::storage::ReadBatch<crate::definition::TestCase>,
    pub ordinal: usize,
    pub projection: CaseProjection,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CaseHeader {
    pub suite_id: String,
    pub case_id: CaseId,
    pub template_id: dokime_domain::model::identity::TemplateId,
    pub metadata: dokime_domain::model::identity::Metadata,
    pub ordinal: usize,
    pub projection: CaseProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseProjection {
    pub status: Option<CaseStatus>,
    pub latest_execution_id: Option<TestExecutionId>,
    pub latest_execution_status: Option<ExecutionStatus>,
    pub attempt_count: u32,
    pub actions: CaseActions,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseTally {
    pub running: u64,
    pub queued: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseActions {
    pub delete: bool,
    pub unavailable_reasons: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl MemorySize for CaseHeader {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.suite_id.memory_size(), std::mem::size_of_val(&self.suite_id)),
            (self.case_id.memory_size(), std::mem::size_of_val(&self.case_id)),
            (self.template_id.memory_size(), std::mem::size_of_val(&self.template_id)),
            (self.metadata.memory_size(), std::mem::size_of_val(&self.metadata)),
            (self.projection.latest_execution_id.memory_size(), std::mem::size_of_val(&self.projection.latest_execution_id)),
            (
                self.projection.actions.unavailable_reasons.memory_size(),
                std::mem::size_of_val(&self.projection.actions.unavailable_reasons),
            ),
        ];
        fields.into_iter().fold(std::mem::size_of_val(self), |bytes, (owned, inline)| bytes.saturating_add(owned.saturating_sub(inline)))
    }
}

impl CaseHeader {
    pub fn new(case: &crate::definition::TestCase, ordinal: usize, projection: CaseProjection) -> Self {
        Self {
            suite_id: case.suite_id.clone(),
            case_id: case.test_case_id.clone(),
            template_id: case.test_case_template_id.clone(),
            metadata: case.metadata.clone(),
            ordinal,
            projection,
        }
    }
}

impl RuntimeHandle {
    pub async fn saved_cycle_cases(&self, id: &dokime_domain::model::identity::TestCycleId) -> RuntimeResult<Option<Vec<CaseHeader>>> {
        let limits = self.query_config().limits();
        tokio::time::timeout(limits.active_time, async {
            let Some(header) = crate::storage::objects::cycle::read(self.storage(), id, ObjectVersion::Latest).await? else {
                return Ok(None);
            };
            let Some(seal) = &header.value.saved else { return Ok(None) };
            if !header.position.covers(&seal.evidence_through)? {
                return Err(Error::conflict("invalid_cycle_membership", "cycle members exceed their saved boundary"));
            }
            let header_bytes = header.value.memory_size() as u64;
            if header_bytes > limits.working_bytes {
                return Err(Error::overload("cycle_inspection_budget", "cycle inspection exceeds the working budget"));
            }
            let members = seal.executions.iter().map(|member| (member.execution_id.to_string(), member.position.sequence)).collect();
            let headers =
                self.local_storage.cycle_case_headers(id.to_string(), members, limits.working_bytes - header_bytes).await.map_err(
                    |error| match error {
                        error @ crate::storage::Error::Budget { .. } => Error::overload("cycle_inspection_budget", error.to_string()),
                        error => error.into(),
                    },
                )?;
            if headers.len() != seal.executions.len() {
                return Err(Error::conflict(
                    "case_index_missing",
                    format!(
                        "cycle case headers differ from sealed membership: expected {}, found {}",
                        seal.executions.len(),
                        headers.len()
                    ),
                ));
            }
            let mut cases: Vec<CaseHeader> = Vec::new();
            let mut retained = header_bytes.saturating_add(std::mem::size_of_val(&cases) as u64);
            let mut ordinals = BTreeMap::new();
            let reason = header.value.phase.ensure_active().err().map(str::to_owned).into_iter().collect::<Vec<_>>();
            for (mut case, member) in headers.into_iter().zip(&seal.executions) {
                if case.projection.latest_execution_id.as_ref() != Some(&member.execution_id) {
                    return Err(Error::conflict("invalid_case_membership", "case header differs from its owning cycle"));
                }
                if let Some(&ordinal) = ordinals.get(&case.case_id) {
                    let first: &mut CaseHeader = &mut cases[ordinal];
                    first.projection.status = case.projection.status;
                    first.projection.latest_execution_id = case.projection.latest_execution_id;
                    first.projection.latest_execution_status = case.projection.latest_execution_status;
                    first.projection.attempt_count = first
                        .projection
                        .attempt_count
                        .checked_add(1)
                        .ok_or_else(|| Error::overload("case_attempt_count", "case attempt count overflows"))?;
                    continue;
                }
                let ordinal = cases.len();
                ordinals.insert(case.case_id.clone(), ordinal);
                case.ordinal = ordinal;
                case.projection.actions = CaseActions {
                    delete: false,
                    unavailable_reasons: reason.clone(),
                };
                retained = retained.saturating_add(case.memory_size() as u64);
                if retained > limits.working_bytes {
                    return Err(Error::overload("cycle_inspection_budget", "cycle inspection exceeds the working budget"));
                }
                cases.push(case);
            }
            Ok(Some(cases))
        })
        .await
        .map_err(|_| Error::timeout("cycle_cases_deadline", "cycle cases exceeded their active-work deadline"))?
    }

    pub async fn inspect_case(
        &self, cycle_id: &dokime_domain::model::identity::TestCycleId, case_id: &CaseId,
    ) -> RuntimeResult<CaseInspection> {
        let snapshot = self.watch_state().latest();
        if let Some(cycle) = snapshot.state().cycle_view(cycle_id) {
            let (ordinal, case) = cycle
                .cases()
                .enumerate()
                .find(|(_, case)| &case.test_case_id == case_id)
                .ok_or_else(|| Error::not_found("case_missing", "case is absent from the requested cycle"))?;
            let case = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(case.clone()))?;
            let projection = case_projection(snapshot.state(), &cycle, case_id);
            return Ok(CaseInspection { snapshot, case, ordinal, projection });
        }
        let cycle = crate::storage::objects::cycle::read(self.storage(), cycle_id, ObjectVersion::Latest)
            .await?
            .ok_or_else(|| Error::not_found("cycle_missing", "cycle is unavailable"))?;
        let seal = cycle.value.saved.as_ref().ok_or_else(|| Error::conflict("cycle_unsealed", "historical cycle is unsealed"))?;
        seal.validate(&cycle.value).map_err(crate::storage::Error::from)?;
        let cases = self.local_storage.cycle_execution_cases(cycle.position.clone(), cycle_id.to_string()).await?;
        let mut seen = std::collections::BTreeSet::new();
        let mut ordinal = None;
        let mut case = None;
        let mut latest = None;
        let mut count = 0u32;
        for (index, id) in cycle.value.execution_ids.iter().enumerate() {
            let owner = cases.get(id).ok_or_else(|| Error::conflict("case_index_missing", "cycle member has no case ownership index"))?;
            if seen.insert(owner) && owner == case_id {
                ordinal = Some(seen.len() - 1);
            }
            if owner != case_id {
                continue;
            }
            let revision = &seal.executions[index];
            let member = crate::storage::objects::execution::read(self.storage(), id, ObjectVersion::Exact(revision.version.clone()))
                .await?
                .ok_or_else(|| Error::not_found("case_attempt_missing", "case attempt is unavailable"))?;
            if member.position != revision.position || member.value.test_case_id != *case_id || member.value.test_cycle_id != *cycle_id {
                return Err(Error::conflict("invalid_case_membership", "case attempt differs from its owning cycle"));
            }
            if cycle.value.memory_size().saturating_add(member.value.memory_size()) as u64 > self.query_config().limits().working_bytes {
                return Err(Error::overload("case_inspection_budget", "case inspection exceeds the working budget"));
            }
            if case.is_none() {
                case = Some(harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(member.value.snapshot.case().clone()))?);
            }
            latest = Some(member);
            count = count.checked_add(1).ok_or_else(|| Error::overload("case_attempt_count", "case attempt count overflows"))?;
        }
        let case = case.ok_or_else(|| Error::not_found("case_missing", "case is absent from the requested cycle"))?;
        let latest = latest.as_ref().map(|member| &member.value);
        let projection = CaseProjection {
            status: Some(status(latest)),
            latest_execution_id: latest.map(|member| member.test_execution_id.clone()),
            latest_execution_status: latest.map(|member| member.status.clone()),
            attempt_count: count,
            actions: CaseActions {
                delete: false,
                unavailable_reasons: cycle.value.phase.ensure_active().err().map(str::to_owned).into_iter().collect(),
            },
        };
        Ok(CaseInspection {
            snapshot,
            case,
            ordinal: ordinal.expect("selected case has indexed membership"),
            projection,
        })
    }
}

/* === Public Functions === */

pub fn case_projection(_state: &RuntimeState, cycle: &CycleView, test_case_id: &CaseId) -> CaseProjection {
    let attempts = attempts(cycle, test_case_id);
    let latest = attempts.last().copied();
    let mut actions = CaseActions {
        delete: true,
        unavailable_reasons: Vec::new(),
    };
    admit(&mut actions.delete, &mut actions.unavailable_reasons, ensure_case_deletable(cycle, test_case_id));

    CaseProjection {
        status: Some(status(latest)),
        latest_execution_id: latest.map(|execution| execution.test_execution_id.clone()),
        latest_execution_status: latest.map(|execution| execution.status.clone()),
        attempt_count: attempts.len() as u32,
        actions,
    }
}

pub fn case_tally(state: &RuntimeState, cycle: &TestCycle) -> CaseTally {
    let mut latest = BTreeMap::new();
    for execution in state.executions_for_cycle(cycle) {
        latest.insert(&execution.test_case_id, execution);
    }

    let mut tally = CaseTally::default();
    for execution in latest.into_values() {
        match status(Some(execution)) {
            CaseStatus::Running => tally.running += 1,
            CaseStatus::Planned | CaseStatus::Queued => tally.queued += 1,
            _ => {}
        }
    }
    tally
}

pub fn historical_case_projection(cycle: &CycleView, test_case_id: &CaseId) -> CaseProjection {
    let (latest, attempt_count) = historical_attempts(cycle, test_case_id);
    CaseProjection {
        status: latest.map(|execution| status(Some(execution))),
        latest_execution_id: latest.map(|execution| execution.test_execution_id.clone()),
        latest_execution_status: latest.map(|execution| execution.status.clone()),
        attempt_count,
        actions: CaseActions {
            unavailable_reasons: vec![HISTORICAL.to_owned()],
            ..Default::default()
        },
    }
}

/* === Private Functions === */

fn historical_attempts<'a>(cycle: &'a CycleView, test_case_id: &CaseId) -> (Option<&'a TestExecution>, u32) {
    cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .fold((None, 0u32), |(_, count), execution| (Some(execution), count.wrapping_add(1)))
}

fn attempts<'a>(cycle: &'a CycleView, test_case_id: &CaseId) -> Vec<&'a TestExecution> {
    cycle.test_executions.iter().filter(|execution| &execution.test_case_id == test_case_id).collect()
}

pub(crate) fn status(latest: Option<&TestExecution>) -> CaseStatus {
    let Some(execution) = latest else {
        return CaseStatus::Planned;
    };
    // A materialized attempt nobody dispatched is the case's queued work, not
    // a skip: `Skipped` is how the runtime spells "planned" on an attempt.
    if execution.status == ExecutionStatus::Skipped && execution.started_at.is_none() && execution.finished_at.is_none() {
        return CaseStatus::Queued;
    }
    match execution.status {
        ExecutionStatus::Skipped => CaseStatus::Skipped,
        ExecutionStatus::Invalidated => CaseStatus::Invalidated,
        ExecutionStatus::Pending => CaseStatus::Queued,
        ExecutionStatus::Running => CaseStatus::Running,
        ExecutionStatus::Passed => CaseStatus::Passed,
        ExecutionStatus::Failed => CaseStatus::Failed,
        ExecutionStatus::Blocked => CaseStatus::Blocked,
        ExecutionStatus::Cancelled => CaseStatus::Cancelled,
        ExecutionStatus::Interrupted => CaseStatus::Interrupted,
    }
}

fn admit(allowed: &mut bool, reasons: &mut Vec<String>, outcome: Result<(), String>) {
    if let Err(reason) = outcome {
        *allowed = false;
        if !reasons.contains(&reason) {
            reasons.push(reason);
        }
    }
}

/* === Tests === */

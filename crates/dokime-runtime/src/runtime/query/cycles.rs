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
use std::cmp::Ordering;
use std::sync::Arc;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectRead, ObjectVersion, query::ObjectVersions},
};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::queries::RuntimeStateSnapshot;
use crate::state::RuntimeState;
use crate::storage::objects::cycle::Cycle;
use crate::{
    RuntimeHandle,
    runtime::{Error, Result},
};
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::identity::{TestCycleId, TestExecutionId};
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CycleListPage {
    pub cycles: Vec<ListedCycle>,
    pub next: Option<String>,
}

pub struct ListedCycle {
    pub cycle: TestCycle,
    pub case_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleFilter {
    pub text: String,
    pub statuses: Vec<CycleStatus>,
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    pub archive: super::executions::ArchiveScope,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleOrder {
    pub key: CycleOrderKey,
    pub descending: bool,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleOrderKey {
    #[default]
    UpdatedAt,

    CreatedAt,
    Label,
}

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn list_cycle_headers(
        &self, state: &RuntimeState, filter: &CycleFilter, order: CycleOrder, after: Option<&str>, limit: usize,
    ) -> Result<CycleListPage> {
        let limits = self.query_config().limits();
        tokio::time::timeout(limits.active_time, async {
            if limit == 0 || limit > limits.max_page_rows as usize {
                return Err(Error::validation("invalid_cycle_page", "invalid cycle page size", vec![]));
            }
            let after =
                match after {
                    None => None,
                    Some(id) => {
                        let id = TestCycleId::from_uuid(id.to_owned())
                            .map_err(|_| Error::validation("invalid_cycle_page", "invalid continuation cycle", vec![]))?;
                        let cycle = if let Some(cycle) = state.cycles.get(&id) {
                            Some(cycle.clone())
                        } else {
                            crate::storage::objects::cycle::read(self.storage(), &id, ObjectVersion::Latest)
                                .await?
                                .filter(|row| row.value.saved.is_some())
                                .map(|row| row.value.clone())
                        };
                        Some(cycle.filter(|cycle| matches_filter(state, cycle, filter)).ok_or_else(|| {
                            Error::validation("invalid_cycle_page", "after cycle ID is not in the filtered result", vec![])
                        })?)
                    }
                };
            let mut selected = Vec::new();
            let mut scanned = 0u64;
            let mut bytes = 0u64;
            {
                let mut retain = |cycle: &TestCycle, include: bool| -> Result<()> {
                    scanned = scanned.saturating_add(1);
                    bytes = bytes.saturating_add(cycle.memory_size() as u64);
                    if scanned > limits.scanned_rows || bytes > limits.scanned_bytes {
                        return Err(Error::overload("cycle_scan_budget", "cycle list exceeds its scan budget"));
                    }
                    if !include
                        || !matches_filter(state, cycle, filter)
                        || after.as_ref().is_some_and(|after| compare(cycle, after, order) != Ordering::Greater)
                    {
                        return Ok(());
                    }
                    selected.push(cycle.clone());
                    selected.sort_by(|left, right| compare(left, right, order));
                    selected.truncate(limit + 1);
                    let retained = selected.iter().fold(0usize, |bytes, cycle| bytes.saturating_add(cycle.memory_size()));
                    if retained as u64 > limits.working_bytes {
                        return Err(Error::overload("cycle_page_budget", "cycle headers exceed the working budget"));
                    }
                    Ok(())
                };
                for cycle in state.cycles.values() {
                    retain(cycle, true)?;
                }
                let mut reader = self
                    .storage()
                    .read_objects::<Cycle>(ObjectRead {
                        at: self.storage().cursor_saved(),
                        key: None,
                        versions: ObjectVersions::Latest,
                    })
                    .await?;
                while let Some(rows) = reader.next().await? {
                    let headers = rows.try_map(|rows| {
                        rows.into_iter().map(|row| row.value.try_into()).collect::<crate::storage::Result<Vec<TestCycle>>>()
                    })?;
                    for cycle in &*headers {
                        // Live State Supersedes Its Checkpoint, Even When The Live Row Does Not Match.
                        let include = !state.cycles.contains_key(&cycle.test_cycle_id) && cycle.saved.is_some();
                        retain(cycle, include)?;
                    }
                }
            }
            let more = selected.len() > limit;
            selected.truncate(limit);
            let next = more.then(|| selected.last().expect("nonempty continued cycle page").test_cycle_id.to_string());
            let mut cycles = Vec::with_capacity(selected.len());
            for cycle in selected {
                let case_count = if state.cycles.contains_key(&cycle.test_cycle_id) {
                    state.cases_for_cycle(&cycle).count() as u64
                } else {
                    self.local_storage.history_cycle_case_count(cycle.test_cycle_id.to_string()).await?
                };
                cycles.push(ListedCycle { cycle, case_count });
            }
            Ok(CycleListPage { cycles, next })
        })
        .await
        .map_err(|_| Error::timeout("cycle_list_deadline", "cycle list exceeded its active-work deadline"))?
    }

    // Historical Inspection Is A Read-Only Projection; Never Reinsert Retired Work Into Managed State.
    pub async fn inspect_cycle(&self, id: &TestCycleId) -> Result<RuntimeStateSnapshot> {
        let snapshot = self.watch_state().latest();
        if snapshot.state().cycles.contains_key(id) {
            return Ok(snapshot);
        }
        let Some(header) = crate::storage::objects::cycle::read(self.storage(), id, ObjectVersion::Latest).await? else {
            return Ok(snapshot);
        };
        let Some(seal) = &header.value.saved else { return Ok(snapshot) };
        let mut state = RuntimeState::new(snapshot.state().session_id.clone());
        state.record_namespace = snapshot.state().record_namespace.clone();
        state.archived_cycles = snapshot.state().archived_cycles.clone();
        state.archived_executions = snapshot.state().archived_executions.clone();
        let mut bytes = header.value.memory_size();
        if bytes as u64 > self.query_config().limits().working_bytes {
            return Err(Error::overload("cycle_inspection_budget", "cycle inspection exceeds the working budget"));
        }
        for revision in &seal.executions {
            let member = crate::storage::objects::execution::read(
                self.storage(),
                &revision.execution_id,
                ObjectVersion::Exact(revision.version.clone()),
            )
            .await?
            .ok_or_else(|| Error::not_found("cycle_member_missing", "saved cycle member is unavailable"))?;
            bytes = bytes.saturating_add(member.value.memory_size());
            if bytes as u64 > self.query_config().limits().working_bytes {
                return Err(Error::overload("cycle_inspection_budget", "cycle inspection exceeds the working budget"));
            }
            state.executions.insert(member.value.test_execution_id.clone(), member.value.clone());
        }
        state.cycles.insert(id.clone(), header.value.clone());
        state.validate_execution_ownership().map_err(|reason| Error::conflict("invalid_cycle_membership", reason))?;
        Ok(RuntimeStateSnapshot::new(Arc::new(state), snapshot.revision()))
    }

    pub async fn inspect_execution(&self, id: &TestExecutionId) -> Result<RuntimeStateSnapshot> {
        let snapshot = self.watch_state().latest();
        if snapshot.state().executions.contains_key(id) {
            return Ok(snapshot);
        }
        let Some(member) = crate::storage::objects::execution::read(self.storage(), id, ObjectVersion::Latest).await? else {
            return Ok(snapshot);
        };
        if member.value.saved.is_none() {
            return Ok(snapshot);
        }
        let cycle_id = member.value.test_cycle_id.clone();
        drop(member);
        self.inspect_cycle(&cycle_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn matches_filter(state: &RuntimeState, cycle: &TestCycle, filter: &CycleFilter) -> bool {
    matches_cycle(cycle, state.archived_cycles.contains(&cycle.test_cycle_id), filter)
}

pub(crate) fn matches_cycle(cycle: &TestCycle, archived: bool, filter: &CycleFilter) -> bool {
    if !filter.archive.admits(archived) {
        return false;
    }
    if !filter.statuses.is_empty() && !filter.statuses.contains(&cycle.lifecycle) {
        return false;
    }
    if !matches_text(cycle, &filter.text) {
        return false;
    }
    within(&cycle.created_at, filter.created_after.as_ref(), filter.created_before.as_ref())
}

fn matches_text(cycle: &TestCycle, text: &str) -> bool {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    let contains = |candidate: &str| candidate.to_lowercase().contains(&needle);
    contains(&cycle.metadata.label)
        || cycle.metadata.description.as_deref().is_some_and(contains)
        || contains(cycle.metadata.identifier.as_str())
}

fn within(value: &Timestamp, after: Option<&Timestamp>, before: Option<&Timestamp>) -> bool {
    if after.is_none() && before.is_none() {
        return true;
    }
    let Some(instant) = value.to_datetime().ok() else {
        return false;
    };
    let admits = |bound: Option<&Timestamp>, lower: bool| match bound {
        None => true,
        Some(bound) => match bound.to_datetime() {
            Err(_) => false,
            Ok(bound) if lower => instant >= bound,
            Ok(bound) => instant <= bound,
        },
    };
    admits(after, true) && admits(before, false)
}

fn compare(left: &TestCycle, right: &TestCycle, order: CycleOrder) -> Ordering {
    let keyed = match order.key {
        CycleOrderKey::UpdatedAt => instants(&left.updated_at, &right.updated_at),
        CycleOrderKey::CreatedAt => instants(&left.created_at, &right.created_at),
        CycleOrderKey::Label => left.metadata.label.to_lowercase().cmp(&right.metadata.label.to_lowercase()),
    };
    let keyed = if order.descending { keyed.reverse() } else { keyed };
    keyed.then_with(|| left.test_cycle_id.as_str().cmp(right.test_cycle_id.as_str()))
}

fn instants(left: &Timestamp, right: &Timestamp) -> Ordering {
    match (left.to_datetime().ok(), right.to_datetime().ok()) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/* === Tests === */

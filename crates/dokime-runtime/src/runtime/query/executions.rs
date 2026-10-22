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

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::execution::storage::ExecutionMetadata;
use crate::state::RuntimeState;
use crate::transactions::{ControlExecution, CycleControlAction};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionControls {
    pub request_id: Option<CommandId>,
    pub mode: dokime_domain::model::execution::ExecutionMode,
    pub paused: bool,
    pub stop_requested: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_stop: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFilter {
    pub text: String,
    pub statuses: Vec<ExecutionStatus>,
    pub template_ids: Vec<String>,
    pub cycle_ids: Vec<String>,
    pub started_after: Option<Timestamp>,
    pub started_before: Option<Timestamp>,
    pub archive: ArchiveScope,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub key: ExecutionOrderKey,
    pub descending: bool,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveScope {
    #[default]
    Unarchived,

    Archived,
    All,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionOrderKey {
    #[default]
    StartedAt,

    FinishedAt,
    Title,
}

/* === Traits === */

/* === Implementations === */

impl ArchiveScope {
    pub fn admits(self, archived: bool) -> bool {
        match self {
            Self::Unarchived => !archived,
            Self::Archived => archived,
            Self::All => true,
        }
    }
}

impl ExecutionFilter {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/* === Public Functions === */

pub fn execution_controls(state: &RuntimeState, id: &TestExecutionId) -> Option<ExecutionControls> {
    let execution = state.executions.get(id)?;
    if execution.saved.is_some() {
        return None;
    }
    let request = ControlExecution::owning_request(state, id);
    if state.cycles.get(&execution.test_cycle_id)?.phase != dokime_domain::model::execution::result::CyclePhase::Active {
        return None;
    }
    let accepts = |action| {
        ControlExecution { test_execution_id: id.clone(), action, occurred_at: crate::model::time::now_utc() }.request(state).is_ok()
    };

    Some(ExecutionControls {
        request_id: request.map(|request| request.summary.request_id.clone()),
        mode: request.map_or(execution.mode, |request| request.summary.mode),
        paused: request.is_some_and(|request| request.summary.paused),
        stop_requested: request.is_some_and(|request| request.summary.stop_requested),
        can_pause: accepts(CycleControlAction::Pause),
        can_resume: accepts(CycleControlAction::Resume),
        can_stop: !state.shutting_down,
    })
}

pub fn matches(metadata: &ExecutionMetadata, filter: &ExecutionFilter) -> bool {
    if !filter.statuses.is_empty() && !filter.statuses.contains(&metadata.status) {
        return false;
    }
    if !filter.cycle_ids.is_empty() && !filter.cycle_ids.iter().any(|want| want == metadata.cycle_id.as_str()) {
        return false;
    }
    if !filter.template_ids.is_empty() && !metadata.template_id.as_ref().is_some_and(|id| filter.template_ids.contains(id)) {
        return false;
    }
    if !matches_text(metadata, &filter.text) {
        return false;
    }
    started_within(metadata, filter)
}

pub fn compare(left: &ExecutionMetadata, right: &ExecutionMetadata, order: ExecutionOrder) -> Ordering {
    let keyed = match order.key {
        ExecutionOrderKey::StartedAt => instants(left.started_at.as_ref(), right.started_at.as_ref()),
        ExecutionOrderKey::FinishedAt => instants(left.finished_at.as_ref(), right.finished_at.as_ref()),
        ExecutionOrderKey::Title => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
    };
    let keyed = if order.descending { keyed.reverse() } else { keyed };
    keyed.then_with(|| left.execution_id.as_str().cmp(right.execution_id.as_str()))
}

/* === Private Functions === */

fn matches_text(metadata: &ExecutionMetadata, text: &str) -> bool {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    let contains = |candidate: &str| candidate.to_lowercase().contains(&needle);
    contains(&metadata.title) || contains(metadata.execution_id.as_str()) || metadata.suite_label.as_deref().is_some_and(contains)
}

fn started_within(metadata: &ExecutionMetadata, filter: &ExecutionFilter) -> bool {
    if filter.started_after.is_none() && filter.started_before.is_none() {
        return true;
    }
    let Some(instant) = metadata.started_at.as_ref().and_then(|value| value.to_datetime().ok()) else {
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
    admits(filter.started_after.as_ref(), true) && admits(filter.started_before.as_ref(), false)
}

fn instants(left: Option<&Timestamp>, right: Option<&Timestamp>) -> Ordering {
    match (left.and_then(|value| value.to_datetime().ok()), right.and_then(|value| value.to_datetime().ok())) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/* === Tests === */

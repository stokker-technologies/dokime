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
use crate::execution::ExecutionStatus;
use crate::execution::storage::ExecutionMetadata;
use crate::state::RuntimeState;
use crate::transactions::{ControlExecution, CycleControlAction};
use dokime::core::{CommandId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionControls {
    pub request_id: CommandId,
    pub mode: crate::execution::ExecutionMode,
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub key: ExecutionOrderKey,
    pub descending: bool,
}

/* === Enums === */
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
impl ExecutionFilter {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/* === Public Functions === */
pub fn execution_controls(state: &RuntimeState, id: &TestExecutionId) -> Option<ExecutionControls> {
    if state.executions.get(id)?.finished_at.is_some() {
        return None;
    }
    let request = ControlExecution::owning_request(state, id)?;
    let accepts = |action| {
        ControlExecution {
            test_execution_id: id.clone(),
            action,
            occurred_at: Timestamp::now_utc(),
        }
        .request(state)
        .is_ok()
    };

    Some(ExecutionControls {
        request_id: request.summary.request_id.clone(),
        mode: request.summary.mode,
        paused: request.summary.paused,
        stop_requested: request.summary.stop_requested
            || request.stopped_execution_ids.contains(id),
        can_pause: accepts(CycleControlAction::Pause),
        can_resume: accepts(CycleControlAction::Resume),
        can_stop: accepts(CycleControlAction::Stop),
    })
}

pub(crate) fn selection_memory(metadata: &ExecutionMetadata, filter: &ExecutionFilter) -> usize {
    let candidate = [
        Some(metadata.title.as_str()),
        Some(metadata.execution_id.as_str()),
        metadata.suite_label.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(lowercase_memory)
    .max()
    .unwrap_or(0);
    lowercase_memory(filter.text.trim()).saturating_add(candidate)
}

pub(crate) fn ordering_memory(metadata: &ExecutionMetadata, order: ExecutionOrder) -> usize {
    if order.key == ExecutionOrderKey::Title {
        lowercase_memory(&metadata.title).saturating_mul(2)
    } else {
        0
    }
}

pub fn matches(metadata: &ExecutionMetadata, filter: &ExecutionFilter) -> bool {
    if !filter.statuses.is_empty() && !filter.statuses.contains(&metadata.status) {
        return false;
    }
    if !filter.cycle_ids.is_empty()
        && !filter
            .cycle_ids
            .iter()
            .any(|want| want == metadata.cycle_id.as_str())
    {
        return false;
    }
    if !filter.template_ids.is_empty()
        && !metadata
            .template_id
            .as_ref()
            .is_some_and(|id| filter.template_ids.contains(id))
    {
        return false;
    }
    if !matches_text(metadata, &filter.text) {
        return false;
    }
    started_within(metadata, filter)
}

pub fn compare(
    left: &ExecutionMetadata,
    right: &ExecutionMetadata,
    order: ExecutionOrder,
) -> Ordering {
    let keyed = match order.key {
        ExecutionOrderKey::StartedAt => {
            instants(left.started_at.as_ref(), right.started_at.as_ref())
        }
        ExecutionOrderKey::FinishedAt => {
            instants(left.finished_at.as_ref(), right.finished_at.as_ref())
        }
        ExecutionOrderKey::Title => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
    };
    let keyed = if order.descending {
        keyed.reverse()
    } else {
        keyed
    };
    keyed.then_with(|| left.execution_id.as_str().cmp(right.execution_id.as_str()))
}

/* === Private Functions === */
fn lowercase_memory(text: &str) -> usize {
    let mut capacity = text.len();
    let mut length = 0usize;
    let mut peak = capacity;
    for character in text.chars().flat_map(char::to_lowercase) {
        length = length.saturating_add(character.len_utf8());
        if length > capacity {
            let next = capacity.saturating_mul(2).max(length).max(8);
            peak = peak.max(capacity.saturating_add(next));
            capacity = next;
        }
    }
    std::mem::size_of::<String>().saturating_add(peak)
}

fn matches_text(metadata: &ExecutionMetadata, text: &str) -> bool {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    let contains = |candidate: &str| candidate.to_lowercase().contains(&needle);
    contains(&metadata.title)
        || contains(metadata.execution_id.as_str())
        || metadata.suite_label.as_deref().is_some_and(contains)
}

fn started_within(metadata: &ExecutionMetadata, filter: &ExecutionFilter) -> bool {
    if filter.started_after.is_none() && filter.started_before.is_none() {
        return true;
    }
    let Some(instant) = metadata
        .started_at
        .as_ref()
        .and_then(|value| value.to_datetime().ok())
    else {
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
    match (
        left.and_then(|value| value.to_datetime().ok()),
        right.and_then(|value| value.to_datetime().ok()),
    ) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/* === Tests === */

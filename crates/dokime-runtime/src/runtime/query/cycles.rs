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
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::TestCycle;
use crate::state::RuntimeState;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CyclePageError {
    UnknownContinuation,
}

/* === Traits === */

/* === Implementations === */
impl std::fmt::Display for CyclePageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownContinuation => {
                write!(formatter, "after cycle ID is not in the filtered result")
            }
        }
    }
}

/* === Public Functions === */
pub fn select_cycles<'a>(
    state: &'a RuntimeState,
    filter: &CycleFilter,
    order: CycleOrder,
) -> Vec<&'a TestCycle> {
    let mut selected: Vec<&TestCycle> = state
        .cycles
        .values()
        .filter(|cycle| matches_filter(state, cycle, filter))
        .collect();
    selected.sort_by(|left, right| compare(left, right, order));
    selected
}

pub fn cycle_page<'a>(
    selected: &[&'a TestCycle],
    after: Option<&str>,
    limit: usize,
) -> Result<(Vec<&'a TestCycle>, Option<String>), CyclePageError> {
    let start = match after {
        None => 0,
        Some(id) => {
            selected
                .iter()
                .position(|cycle| cycle.test_cycle_id.as_str() == id)
                .ok_or(CyclePageError::UnknownContinuation)?
                + 1
        }
    };
    let page: Vec<&TestCycle> = selected.iter().skip(start).take(limit).copied().collect();
    let next = (start + page.len() < selected.len())
        .then(|| page.last().map(|cycle| cycle.test_cycle_id.to_string()))
        .flatten();
    Ok((page, next))
}

/* === Private Functions === */
fn matches_filter(state: &RuntimeState, cycle: &TestCycle, filter: &CycleFilter) -> bool {
    if !filter
        .archive
        .admits(state.archived_cycles.contains(&cycle.test_cycle_id))
    {
        return false;
    }
    if !filter.statuses.is_empty() && !filter.statuses.contains(&cycle.lifecycle) {
        return false;
    }
    if !matches_text(cycle, &filter.text) {
        return false;
    }
    within(
        &cycle.created_at,
        filter.created_after.as_ref(),
        filter.created_before.as_ref(),
    )
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
        CycleOrderKey::Label => left
            .metadata
            .label
            .to_lowercase()
            .cmp(&right.metadata.label.to_lowercase()),
    };
    let keyed = if order.descending {
        keyed.reverse()
    } else {
        keyed
    };
    keyed.then_with(|| {
        left.test_cycle_id
            .as_str()
            .cmp(right.test_cycle_id.as_str())
    })
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

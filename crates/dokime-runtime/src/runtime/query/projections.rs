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

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::{HealthItem, OperatorNote};
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use crate::records::ResourceRecord;
use crate::traceability::TraceabilitySnapshot;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::time::Timestamp;
use crate::prompts::PromptInstance;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeResourceFrame {
    pub records: Vec<ResourceRecord>,
    pub total_records: usize,
    pub returned_records: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimePromptFrame {
    pub prompts: Vec<PromptInstance>,
    pub total_prompts: usize,
    pub returned_prompts: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHealthFrame {
    pub items: Vec<HealthItem>,
    pub total_items: usize,
    pub returned_items: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeTraceabilityFrame {
    pub snapshots: Vec<TraceabilitySnapshot>,
    pub total_snapshots: usize,
    pub returned_snapshots: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeNoteFrame {
    pub notes: Vec<OperatorNote>,
    pub total_notes: usize,
    pub returned_notes: usize,
    pub truncated: bool,
}

struct Reduced<T> {
    items: Vec<T>,
    total: usize,
    returned: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<T> Reduced<T> {
    fn truncated(&self) -> bool {
        self.returned < self.total
    }
}

impl From<Reduced<ResourceRecord>> for RuntimeResourceFrame {
    fn from(reduced: Reduced<ResourceRecord>) -> Self {
        Self {
            truncated: reduced.truncated(),
            records: reduced.items,
            total_records: reduced.total,
            returned_records: reduced.returned,
        }
    }
}

impl From<Reduced<PromptInstance>> for RuntimePromptFrame {
    fn from(reduced: Reduced<PromptInstance>) -> Self {
        Self {
            truncated: reduced.truncated(),
            prompts: reduced.items,
            total_prompts: reduced.total,
            returned_prompts: reduced.returned,
        }
    }
}

impl From<Reduced<HealthItem>> for RuntimeHealthFrame {
    fn from(reduced: Reduced<HealthItem>) -> Self {
        Self {
            truncated: reduced.truncated(),
            items: reduced.items,
            total_items: reduced.total,
            returned_items: reduced.returned,
        }
    }
}

impl From<Reduced<TraceabilitySnapshot>> for RuntimeTraceabilityFrame {
    fn from(reduced: Reduced<TraceabilitySnapshot>) -> Self {
        Self {
            truncated: reduced.truncated(),
            snapshots: reduced.items,
            total_snapshots: reduced.total,
            returned_snapshots: reduced.returned,
        }
    }
}

impl From<Reduced<OperatorNote>> for RuntimeNoteFrame {
    fn from(reduced: Reduced<OperatorNote>) -> Self {
        Self {
            truncated: reduced.truncated(),
            notes: reduced.items,
            total_notes: reduced.total,
            returned_notes: reduced.returned,
        }
    }
}

/* === Public Functions === */
pub(crate) fn resource_frame<'a>(
    records: impl Iterator<Item = &'a ResourceRecord>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimeResourceFrame {
    windowed(
        records,
        |record| {
            scope.is_none_or(|scope| {
                record
                    .scope
                    .as_ref()
                    .is_some_and(|record_scope| record_scope.is_visible_in(scope))
            })
        },
        |record| &record.updated_at,
        window,
        resolution,
    )
    .into()
}

pub(crate) fn health_frame<'a>(
    items: impl Iterator<Item = &'a HealthItem>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimeHealthFrame {
    windowed(
        items,
        |item| scope.is_none_or(|scope| item.scope.is_visible_in(scope)),
        |item| &item.observed_at,
        window,
        resolution,
    )
    .into()
}

pub(crate) fn prompt_frame<'a>(
    prompts: impl Iterator<Item = &'a PromptInstance>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimePromptFrame {
    windowed(
        prompts,
        |prompt| scope.is_none_or(|scope| prompt.scope.is_visible_in(scope)),
        |prompt| &prompt.created_at,
        window,
        resolution,
    )
    .into()
}

pub(crate) fn traceability_frame<'a>(
    snapshots: impl Iterator<Item = &'a TraceabilitySnapshot>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimeTraceabilityFrame {
    windowed(
        snapshots,
        |snapshot| scope.is_none_or(|scope| snapshot.scope.is_visible_in(scope)),
        |snapshot| &snapshot.created_at,
        window,
        resolution,
    )
    .into()
}

pub(crate) fn note_frame<'a>(
    notes: impl Iterator<Item = &'a OperatorNote>,
    scope: Option<&RecordScope>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimeNoteFrame {
    windowed(
        notes,
        |note| scope.is_none_or(|scope| note.scope.is_visible_in(scope)),
        |note| &note.created_at,
        window,
        resolution,
    )
    .into()
}

/* === Private Functions === */
fn windowed<'a, T: Clone + 'a>(
    items: impl Iterator<Item = &'a T>,
    visible: impl Fn(&T) -> bool,
    timestamp: impl Fn(&T) -> &Timestamp,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> Reduced<T> {
    let items = items
        .filter(|item| visible(item))
        .cloned()
        .collect::<Vec<_>>();
    let reduced = reduce_timestamped_items(items, window, resolution, timestamp, |_| None);
    let returned = reduced.returned.len();

    Reduced {
        items: reduced.returned,
        total: reduced.total,
        returned,
    }
}

/* === Tests === */

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

// Local Imports
use crate::queries::{RuntimeResolution, RuntimeWindow};
use crate::core::{DurationMillis, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct ReducedItems<T> {
    pub(crate) total: usize,
    pub(crate) returned: Vec<T>,
    pub(crate) high_water: Option<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn reduce_timestamped_items<T, F, S>(
    items: Vec<T>,
    window: &RuntimeWindow,
    resolution: &RuntimeResolution,
    timestamp: F,
    sequence: S,
) -> ReducedItems<T>
where
    T: Clone,
    F: Fn(&T) -> &Timestamp,
    S: Fn(&T) -> Option<u64>,
{
    let items = apply_window(items, window, &timestamp, &sequence);
    let total = items.len();
    let high_water = items.iter().filter_map(&sequence).max();
    let returned = apply_resolution(items, resolution);

    ReducedItems {
        total,
        returned,
        high_water,
    }
}

/* === Private Functions === */
fn apply_window<T, F, S>(
    items: Vec<T>,
    window: &RuntimeWindow,
    timestamp: &F,
    sequence: &S,
) -> Vec<T>
where
    F: Fn(&T) -> &Timestamp,
    S: Fn(&T) -> Option<u64>,
{
    match window {
        RuntimeWindow::Full => items,
        RuntimeWindow::Since { cursor } => items
            .into_iter()
            .filter(|item| sequence(item).is_none_or(|sequence| sequence > cursor.0))
            .collect(),
        RuntimeWindow::TimeRange { start, end } => items
            .into_iter()
            .filter(|item| timestamp_in_range(timestamp(item), start, end))
            .collect(),
        RuntimeWindow::Tail { duration } => tail_items(items, *duration, timestamp),
        RuntimeWindow::LastSamples { count } => last_items(items, *count),
    }
}

fn apply_resolution<T: Clone>(items: Vec<T>, resolution: &RuntimeResolution) -> Vec<T> {
    match resolution {
        RuntimeResolution::Raw => items,
        RuntimeResolution::Samples { count } => downsample(items, *count),
        RuntimeResolution::Pixels { width } => downsample(items, *width as usize),
        RuntimeResolution::Summary => last_items(items, 1),
    }
}

fn timestamp_in_range(timestamp: &Timestamp, start: &Timestamp, end: &Timestamp) -> bool {
    start.millis_until(timestamp).is_some() && timestamp.millis_until(end).is_some()
}

fn tail_items<T>(
    items: Vec<T>,
    duration: DurationMillis,
    timestamp: impl Fn(&T) -> &Timestamp,
) -> Vec<T> {
    let Some(end) = items.last().map(|item| timestamp(item).clone()) else {
        return items;
    };

    items
        .into_iter()
        .filter(|item| {
            timestamp(item)
                .millis_until(&end)
                .is_some_and(|elapsed| elapsed.0 <= duration.0)
        })
        .collect()
}

fn last_items<T>(items: Vec<T>, count: usize) -> Vec<T> {
    if count == 0 || items.is_empty() {
        return Vec::new();
    }

    items
        .into_iter()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn downsample<T>(items: Vec<T>, count: usize) -> Vec<T> {
    if items.len() <= count {
        return items;
    }
    let mut selected = harmos::signal::decimate(items.len(), count)
        .into_iter()
        .peekable();
    items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (selected.peek() == Some(&index)).then(|| {
                selected.next();
                item
            })
        })
        .collect()
}

/* === Tests === */

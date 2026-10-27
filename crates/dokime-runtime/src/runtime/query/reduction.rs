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
use dokime_domain::model::time::DurationMillis;
use dokime_domain::model::time::Timestamp;

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
    items: Vec<T>, window: &RuntimeWindow, resolution: &RuntimeResolution, timestamp: F, sequence: S,
) -> ReducedItems<T>
where
    F: Fn(&T) -> &Timestamp,
    S: Fn(&T) -> Option<u64>,
{
    let items = apply_window(items, window, &timestamp, &sequence);
    let total = items.len();
    let high_water = items.iter().filter_map(&sequence).max();
    let returned = apply_resolution(items, resolution);

    ReducedItems { total, returned, high_water }
}

/* === Private Functions === */

fn apply_window<T, F, S>(mut items: Vec<T>, window: &RuntimeWindow, timestamp: &F, sequence: &S) -> Vec<T>
where
    F: Fn(&T) -> &Timestamp,
    S: Fn(&T) -> Option<u64>,
{
    match window {
        RuntimeWindow::Full => items,
        RuntimeWindow::Since { cursor } => {
            items.retain(|item| sequence(item).is_none_or(|sequence| sequence > cursor.0));
            items
        }
        RuntimeWindow::TimeRange { start, end } => {
            items.retain(|item| timestamp_in_range(timestamp(item), start, end));
            items
        }
        RuntimeWindow::Tail { duration } => tail_items(items, *duration, timestamp),
        RuntimeWindow::LastSamples { count } => last_items(items, *count),
    }
}

fn apply_resolution<T>(items: Vec<T>, resolution: &RuntimeResolution) -> Vec<T> {
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

fn tail_items<T>(mut items: Vec<T>, duration: DurationMillis, timestamp: impl Fn(&T) -> &Timestamp) -> Vec<T> {
    let Some(end) = items.last().map(|item| timestamp(item).clone()) else {
        return items;
    };

    items.retain(|item| timestamp(item).millis_until(&end).is_some_and(|elapsed| elapsed.0 <= duration.0));
    items
}

fn last_items<T>(mut items: Vec<T>, count: usize) -> Vec<T> {
    if count == 0 || items.is_empty() {
        return Vec::new();
    }

    let remove = items.len().saturating_sub(count);
    items.drain(..remove);
    items
}

fn downsample<T>(mut items: Vec<T>, count: usize) -> Vec<T> {
    if items.len() <= count {
        return items;
    }
    let mut selected = harmos::signal::decimate(items.len(), count).into_iter().peekable();
    let mut index = 0;
    items.retain(|_| {
        let keep = selected.peek() == Some(&index);
        if keep {
            selected.next();
        }
        index += 1;
        keep
    });
    items
}
/* === Tests === */

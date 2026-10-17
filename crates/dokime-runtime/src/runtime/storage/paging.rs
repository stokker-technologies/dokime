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
pub(super) use crate::execution::saved::PageCursor as Cursor;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{runtime::Error, runtime::Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Positioned<T> {
    pub(super) position: u64,
    pub(super) captured: T,
}

pub(super) struct State<T> {
    pub(super) rows: Vec<T>,
    pub(super) last: u64,
    pub(super) more: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn page_limit(limit: Option<usize>, noun: &str) -> Result<usize> {
    let limit = limit.unwrap_or(100);
    if !(1..=1000).contains(&limit) {
        return Err(Error::Harmos(harmos::Error::InvalidInput {
            field: "native evidence page limit",
            reason: match noun {
                "record" => "record page limit must be 1..=1000",
                _ => "telemetry page limit must be 1..=1000",
            },
        }));
    }
    Ok(limit)
}

pub(super) fn update<T: Clone>(
    previous: Option<State<T>>,
    rows: Vec<Positioned<T>>,
    after: u64,
    limit: usize,
) -> Result<State<T>> {
    let mut state = previous.unwrap_or(State {
        rows: Vec::new(),
        last: after,
        more: false,
    });
    for row in rows {
        if row.position <= state.last {
            return Err(Error::Harmos(harmos::Error::InvalidInput {
                field: "native evidence page",
                reason: "saved positions did not advance",
            }));
        }
        if state.rows.len() == limit {
            state.more = true;
            break;
        }
        state.last = row.position;
        state.rows.push(row.captured);
    }
    Ok(state)
}

/* === Private Functions === */

/* === Tests === */

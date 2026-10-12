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
pub(super) use dokime::execution::saved::PageCursor as Cursor;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::{Error, Result};

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
impl<T: MemorySize> MemorySize for Positioned<T> {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.captured
                .memory_size()
                .saturating_sub(std::mem::size_of::<T>()),
        )
    }
}

impl<T: MemorySize> MemorySize for State<T> {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.rows
                .memory_size()
                .saturating_sub(std::mem::size_of_val(&self.rows)),
        )
    }
}

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

pub(super) fn update<T: MemorySize + Clone>(
    previous: Option<&State<T>>,
    rows: &[Positioned<T>],
    after: u64,
    limit: usize,
    capacity: usize,
) -> harmos::Result<State<T>> {
    let old = previous.map_or(&[][..], |state| state.rows.as_slice());
    let mut memory = old
        .iter()
        .try_fold(std::mem::size_of::<State<T>>(), |bytes, record| {
            bytes.checked_add(record.memory_size())
        })
        .ok_or_else(exhausted)?;
    if memory > capacity {
        return Err(exhausted());
    }
    let mut selected = 0;
    let mut last = previous.map_or(after, |state| state.last);
    let mut more = false;
    for row in rows {
        if row.position <= last {
            return Err(harmos::Error::InvalidInput {
                field: "native evidence page",
                reason: "saved positions did not advance",
            });
        }
        let next = memory
            .checked_add(row.captured.memory_size())
            .ok_or_else(exhausted)?;
        if old.len() + selected == limit || next > capacity {
            if old.is_empty() && selected == 0 {
                return Err(exhausted());
            }
            more = true;
            break;
        }
        memory = next;
        selected += 1;
        last = row.position;
    }
    let count = old.len().checked_add(selected).ok_or_else(exhausted)?;
    let mut selected_rows = Vec::new();
    selected_rows
        .try_reserve_exact(count)
        .map_err(|_| exhausted())?;
    if selected_rows.capacity() > count {
        return Err(exhausted());
    }
    selected_rows.extend(old.iter().cloned());
    selected_rows.extend(rows.iter().take(selected).map(|row| row.captured.clone()));
    Ok(State {
        rows: selected_rows,
        last,
        more,
    })
}

/* === Private Functions === */
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "native evidence page",
    }
}

/* === Tests === */

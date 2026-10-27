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

pub(super) use dokime_domain::model::execution::saved::PageCursor as Cursor;

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn page_limit(limit: Option<usize>) -> Result<usize> {
    let default = 100;
    let maximum = 1000;
    let limit = limit.unwrap_or(default);
    if !(1..=maximum).contains(&limit) {
        return Err(Error::Harmos(harmos::Error::InvalidInput {
            field: "native evidence page limit",
            reason: "page limit must be positive and within the configured pagination maximum",
        }));
    }
    Ok(limit)
}
/* === Private Functions === */

/* === Tests === */

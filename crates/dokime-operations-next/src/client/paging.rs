/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Paging

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use sha2::{Digest, Sha256};
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn cursor(context: &Context, selection: String, rows: impl IntoIterator<Item = Vec<u8>>) -> v1::Cursor {
    let mut digest = Sha256::new();
    for row in rows { digest.update((row.len() as u64).to_le_bytes()); digest.update(row); }
    let bytes = digest.finalize();
    v1::Cursor { epoch: context.host.identity().epoch, stream: selection, position: u64::from_le_bytes(bytes[..8].try_into().unwrap()) }
}

pub(super) fn page<T>(mut rows: Vec<T>, requested: u32, after: &str, supplied: Option<&v1::Cursor>, current: &v1::Cursor, maximum: usize, id: impl Fn(&T) -> &str) -> Result<(Vec<T>, Option<String>)> {
    if supplied.is_some_and(|cursor| cursor != current) || (!after.is_empty() && supplied.is_none()) {
        return Err(Error::Conflict("Restart Paging with the Same Selection and a Fresh Cursor".into()));
    }
    let start = if after.is_empty() { 0 } else {
        rows.iter().position(|row| id(row) == after).ok_or_else(|| Error::Invalid("Restart Paging: the Continuation Is Outside the Selection".into()))? + 1
    };
    let limit = super::projection::limit(requested, maximum);
    let remaining = rows.split_off(start);
    let more = remaining.len() > limit;
    let rows: Vec<T> = remaining.into_iter().take(limit).collect();
    let next = more.then(|| id(rows.last().unwrap()).to_owned());
    Ok((rows, next))
}

/* === Private Functions === */

/* === Tests === */

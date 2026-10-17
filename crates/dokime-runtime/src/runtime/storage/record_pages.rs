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
use super::{
    execution::ExecutionView,
    paging::{Cursor, page_limit, update},
    records::RecordReader,
};
use crate::{
    runtime::Result,
    runtime::query::checkpoint_records::{ExecutionRecordPage, ExecutionRecordQuery},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn read(view: &ExecutionView, query: ExecutionRecordQuery) -> Result<ExecutionRecordPage> {
    let limit = page_limit(query.limit, "record")?;
    let after = Cursor::requested(query.after.as_deref())?;
    let mut reader = RecordReader::open_page(view, after).await?;
    let checkpoint = view.checkpoint()?;
    let cursor = Cursor::open(&checkpoint, "records", "", query.after.as_deref())?;
    let mut state = None;
    while let Some(rows) = reader.next_positioned().await? {
        let next = update(state.take(), rows, after, limit)?;
        let complete = next.more;
        state = Some(next);
        if complete {
            break;
        }
    }
    let next = state.as_ref().filter(|state| state.more).map(|state| cursor.encode(state.last)).transpose()?;
    Ok(ExecutionRecordPage {
        checkpoint,
        records: state.map(|state| state.rows).unwrap_or_default(),
        next,
        issues: Vec::new(),
        issues_truncated: false,
    })
}

/* === Private Functions === */

/* === Tests === */

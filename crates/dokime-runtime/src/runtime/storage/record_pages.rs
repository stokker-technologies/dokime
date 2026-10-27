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
    paging::{Cursor, page_limit},
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

pub(crate) async fn read(view: &ExecutionView, query: ExecutionRecordQuery) -> Result<harmos::storage::ReadBatch<ExecutionRecordPage>> {
    let limit = page_limit(query.limit)?;
    let after = Cursor::requested(query.after.as_deref())?;
    let mut reader = RecordReader::open_page(view, after).await?;
    view.revision()?;

    let header = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
        let checkpoint = view.checkpoint()?;
        let cursor = Cursor::open(&checkpoint, "records", "", query.after.as_deref())?;
        Ok((checkpoint, cursor))
    })?;
    let mut records = harmos::storage::ReadBatch::new(Vec::new());
    let mut count = 0usize;
    let mut last = after;
    let mut more = false;
    while let Some(rows) = reader.next_positioned().await? {
        let selected = rows.try_map(|rows| -> Result<_> {
            let mut selected = Vec::new();
            selected
                .try_reserve_exact(rows.len().min(limit - count))
                .map_err(|_| crate::runtime::Error::Operation("record page allocation failed".into()))?;
            for row in rows {
                if row.position <= last {
                    return Err(crate::runtime::Error::Operation("saved record positions did not advance".into()));
                }
                if count == limit {
                    more = true;
                    break;
                }
                last = row.position;
                count += 1;
                selected.push(row.captured);
            }
            Ok(selected)
        })?;
        if !selected.is_empty() {
            records = crate::storage::projection::push(records, selected)?;
        }
        if more {
            break;
        }
    }
    header.try_map(|(checkpoint, cursor)| -> Result<_> {
        let next = more.then(|| cursor.encode(last)).transpose()?;
        Ok(ExecutionRecordPage { checkpoint, records: records.into(), next, issues: Vec::new(), issues_truncated: false })
    })
}

/* === Private Functions === */

/* === Tests === */

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
    context_records::ContextRecord,
    execution::ExecutionView,
    paging::{Cursor, page_limit},
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

pub(crate) async fn read(
    view: &ExecutionView, local: &crate::storage::sqlite::State, query: ExecutionRecordQuery,
) -> Result<harmos::storage::ReadBatch<ExecutionRecordPage>> {
    let limit = page_limit(query.limit)?;
    let after = Cursor::requested(query.after.as_deref())?;
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
    let scope = crate::evidence_runtime::ScopeSelection {
        test_execution_id: Some(view.execution().test_execution_id.clone()),
        ..Default::default()
    };
    let mut scanned = after;
    loop {
        let rows = local.scoped_records(view.evidence_through()?.clone(), scope.clone(), None, scanned, 0, u64::MAX).await?;
        let Some(end) = rows.last() else { break };
        scanned = end.position;
        let selected = harmos::storage::ReadBatch::new(rows).try_map(|rows| -> Result<_> {
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
                selected.push(ContextRecord { epoch: row.epoch, record: row.record });
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
        Ok(ExecutionRecordPage {
            checkpoint,
            records: records.into(),
            next,
            issues: Vec::new(),
            issues_truncated: false,
        })
    })
}

/* === Private Functions === */

/* === Tests === */

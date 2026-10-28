/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use harmos::Identifier;

use super::{
    execution::ExecutionView,
    paging::{Cursor, page_limit},
    telemetry::TelemetryReader,
};
use crate::{
    runtime::Result,
    runtime::query::checkpoint_telemetry::{ExecutionTelemetryPage, ExecutionTelemetryQuery},
};

pub(crate) async fn read(
    view: &ExecutionView, stream: &Identifier, query: ExecutionTelemetryQuery, settings: &harmos::config::SchemaSettings,
) -> Result<harmos::storage::ReadBatch<ExecutionTelemetryPage>> {
    let limit = page_limit(query.limit)?;
    let after = Cursor::requested(query.after.as_deref())?;
    let view = view.snapshot().await?;
    let mut reader = TelemetryReader::open_page(&view, stream, after, settings).await?;
    view.revision()?;

    let header = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
        let checkpoint = view.checkpoint()?;
        let cursor = Cursor::open(&checkpoint, "telemetry", stream.as_str(), query.after.as_deref())?;
        Ok((checkpoint, cursor, stream.as_str().to_owned()))
    })?;
    let mut pages = harmos::storage::ReadBatch::new(Vec::new());
    let mut count = 0_usize;
    let mut last = after;
    let mut more = false;
    while let Some(rows) = reader.next_positioned().await? {
        let selected = rows.try_map(|rows| -> Result<_> {
            let mut selected = Vec::new();
            selected
                .try_reserve_exact(rows.len().min(limit - count))
                .map_err(|_| super::telemetry::invalid("telemetry page allocation failed"))?;
            for row in rows {
                if row.position <= last {
                    return Err(super::telemetry::invalid("saved telemetry positions did not advance"));
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
            pages = crate::storage::projection::push(pages, selected)?;
        }
        if more {
            break;
        }
    }
    header.try_map(|(checkpoint, cursor, table)| -> Result<_> {
        let next = more.then(|| cursor.encode(last)).transpose()?;
        Ok(ExecutionTelemetryPage {
            table,
            checkpoint,
            rows: pages.into(),
            next,
            issues: Vec::new(),
            issues_truncated: false,
        })
    })
}

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
use harmos::Identifier;

// Local Imports
use super::{
    execution::ExecutionView,
    paging::{Cursor, page_limit, update},
    telemetry::TelemetryReader,
};
use crate::{
    runtime::Result,
    runtime::query::checkpoint_telemetry::{ExecutionTelemetryPage, ExecutionTelemetryQuery},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn read(view: &ExecutionView, stream: &Identifier, query: ExecutionTelemetryQuery) -> Result<ExecutionTelemetryPage> {
    let limit = page_limit(query.limit, "telemetry")?;
    let after = Cursor::requested(query.after.as_deref())?;
    let view = view.snapshot().await?;
    let mut reader = TelemetryReader::open_page(&view, stream, after)?;
    let checkpoint = view.checkpoint()?;
    let cursor = Cursor::open(&checkpoint, "telemetry", stream.as_str(), query.after.as_deref())?;
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
    Ok(ExecutionTelemetryPage {
        table: stream.as_str().to_owned(),
        checkpoint,
        rows: state.map(|state| state.rows).unwrap_or_default(),
        next,
        issues: Vec::new(),
        issues_truncated: false,
    })
}

/* === Private Functions === */

/* === Tests === */

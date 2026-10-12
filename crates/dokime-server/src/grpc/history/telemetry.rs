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
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;
use tonic::{Code, Status};

// Local Imports
use super::super::value::projection;
use dokime_app::runtime::query::checkpoint_telemetry::ExecutionTelemetryPage;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn encode(
    page: &ExecutionTelemetryPage,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    // The Original Native Page Remains Charged Separately. These Bounds Admit
    // Every Wire Slot And Copied Field Before Constructing The Target Graph.
    let graph = graph_memory(page)?;
    if graph > allowance.get() {
        return Err(exhausted());
    }
    let mut records = projection::slots(page.rows.len())?;
    for row in &page.rows {
        records.push(super::wire::telemetry_row(row).map_err(original_error)?);
    }
    let mut issues = projection::slots(page.issues.len())?;
    for issue in &page.issues {
        issues.push(v1::HistoryIssue {
            source: issue.source.as_deref().map(projection::text).transpose()?,
            dataset: issue.dataset.as_deref().map(projection::text).transpose()?,
            message: projection::text(&issue.message)?,
        });
    }
    let wire = v1::CheckpointTelemetryPage {
        checkpoint: Some(
            super::wire::checkpoint_wire_ref(&page.checkpoint).map_err(original_error)?,
        ),
        table: projection::text(&page.table)?,
        rows: records,
        next_after: page.next.as_deref().map(projection::text).transpose()?,
        issues,
        issues_truncated: page.issues_truncated,
    };
    let length = wire.encoded_len();
    let peak = graph
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .ok_or_else(exhausted)?;
    if peak > allowance.get() || length > maximum {
        return Err(exhausted());
    }
    let mut bytes = projection::slots(length)?;
    wire.encode(&mut bytes).map_err(|_| invalid())?;
    if bytes.len() != length || bytes.capacity() > length {
        return Err(invalid());
    }
    Ok(bytes)
}

/* === Private Functions === */
fn graph_memory(page: &ExecutionTelemetryPage) -> harmos::Result<usize> {
    let checkpoint = &page.checkpoint;
    let mut bytes = std::mem::size_of::<v1::CheckpointTelemetryPage>();
    for value in [
        page.rows
            .len()
            .checked_mul(std::mem::size_of::<v1::SavedTelemetrySample>())
            .ok_or_else(exhausted)?,
        page.issues
            .len()
            .checked_mul(std::mem::size_of::<v1::HistoryIssue>())
            .ok_or_else(exhausted)?,
        page.table.len(),
        checkpoint.execution_id.as_str().len(),
        harmos::Message::encoded_len(&checkpoint.version),
        checkpoint.position.history.store.as_str().len(),
        checkpoint.position.history.epoch.as_str().len(),
        checkpoint.evidence_through.history.store.as_str().len(),
        checkpoint.evidence_through.history.epoch.as_str().len(),
        page.next.as_ref().map_or(0, String::len),
    ] {
        bytes = bytes.checked_add(value).ok_or_else(exhausted)?;
    }
    for row in &page.rows {
        let row_memory = super::wire::telemetry_memory(row).map_err(original_error)?;
        bytes = bytes.checked_add(row_memory).ok_or_else(exhausted)?;
    }
    for issue in &page.issues {
        for length in [
            issue.message.len(),
            issue.source.as_ref().map_or(0, String::len),
            issue.dataset.as_ref().map_or(0, String::len),
        ] {
            bytes = bytes.checked_add(length).ok_or_else(exhausted)?;
        }
    }
    Ok(bytes)
}
fn original_error(source: Status) -> harmos::Error {
    if source.code() == Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "encode original saved telemetry",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical saved telemetry page",
    }
}
fn invalid() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "canonical saved telemetry page",
        reason: "encoded length differs",
    }
}

/* === Tests === */

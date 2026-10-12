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
use std::mem::{size_of, size_of_val};

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::core::{Metadata, RecordScope, Value};
use crate::document::{
    ArtifactListDescriptor, DocumentCardBody, DocumentCardDefinition, DocumentCardPayload,
    DocumentFilter, PlotDescriptor, PlotInput, TelemetrySummaryGroupPayload,
    TelemetrySummaryTilePayload,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn memory(
    metadata: &Metadata,
    filters: &Vec<DocumentFilter>,
    body: &DocumentCardBody,
    scope: &RecordScope,
    largest_value: usize,
) -> usize {
    let definition = size_of::<DocumentCardDefinition>()
        .saturating_add(owned(metadata))
        .saturating_add(owned(filters))
        .saturating_add(owned(body));
    let mut payload = size_of::<DocumentCardPayload>().saturating_add(owned(metadata));
    match body {
        DocumentCardBody::Text { markdown } => payload = payload.saturating_add(markdown.len()),
        DocumentCardBody::Table { rows } => payload = payload.saturating_add(owned(rows)),
        DocumentCardBody::TelemetrySummary { summary } => {
            payload = payload.saturating_add(
                summary
                    .groups
                    .len()
                    .saturating_mul(size_of::<TelemetrySummaryGroupPayload>()),
            );
            for group in &summary.groups {
                payload = payload
                    .saturating_add(owned(&group.metadata))
                    .saturating_add(
                        group
                            .tiles
                            .len()
                            .saturating_mul(size_of::<TelemetrySummaryTilePayload>()),
                    );
                for tile in &group.tiles {
                    payload = payload
                        .saturating_add(owned(&tile.metadata))
                        .saturating_add(owned(&tile.unit))
                        .saturating_add(super::super::payload::MAX_UNAVAILABLE_REASON_BYTES)
                        .saturating_add(heap(largest_value, size_of::<Value>()));
                }
            }
        }
        DocumentCardBody::Plot { plot } => {
            // PlotInput clones a subset of these declared owned fields, plus
            // selection scope. No available-stream strings are built by current
            // resolution, which supplies the plot builder an empty sample slice.
            let input = size_of::<PlotInput>()
                .saturating_add(owned(plot.as_ref()))
                .saturating_add(owned(scope));
            let descriptor = size_of::<PlotDescriptor>()
                .saturating_add(owned(&plot.metadata))
                .saturating_add(owned(&plot.metadata.identifier))
                .saturating_add(heap(input, size_of::<PlotInput>()));
            // The source PlotInput coexists with its clone in the descriptor.
            payload = payload.saturating_add(input).saturating_add(descriptor);
        }
        DocumentCardBody::Artifacts | DocumentCardBody::Media => {
            let descriptor = size_of::<ArtifactListDescriptor>().saturating_add(owned(scope));
            // The card owns one descriptor; artifact_lists retains another.
            payload = payload
                .saturating_add(descriptor)
                .saturating_add(descriptor);
        }
        DocumentCardBody::Steps
        | DocumentCardBody::Criteria
        | DocumentCardBody::Traceability
        | DocumentCardBody::Notes
        | DocumentCardBody::Records
        | DocumentCardBody::Health
        | DocumentCardBody::TelemetryContext => {}
    }
    definition.saturating_add(payload)
}

/* === Private Functions === */
fn owned<T: MemorySize>(value: &T) -> usize {
    heap(value.memory_size(), size_of_val(value))
}

fn heap(bytes: usize, inline: usize) -> usize {
    if bytes == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_sub(inline)
    }
}

/* === Tests === */

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
use crate::document::{
    DocumentFailure,
    input::{DocumentSelection, Records},
};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::records::evidence::RecordMarker;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn record_markers_for_selection(records: &[RecordEntry], selection: &DocumentSelection) -> crate::Result<Vec<RecordMarker>> {
    selected_markers(Records::Native(records), selection)
}

pub(super) fn selected_markers(records: Records<'_>, selection: &DocumentSelection) -> crate::Result<Vec<RecordMarker>> {
    let count = records.iter().filter(|record| selection.matches_scope(record.scope)).count();
    let mut markers = Vec::new();
    markers.try_reserve_exact(count).map_err(|_| DocumentFailure::Limit)?;
    if markers.capacity() > count {
        return Err(DocumentFailure::Limit.into());
    }
    for record in records.iter() {
        if selection.matches_scope(record.scope) {
            let mut marker = RecordMarker::from_record(record.entry);
            if record.scope != record.entry.scope.as_ref() {
                marker.scope = None;
                marker.scope = record.scope.cloned();
            }
            markers.push(marker);
        }
    }
    Ok(markers)
}

pub(super) fn selection_memory(records: Records<'_>, selection: &DocumentSelection) -> usize {
    records.iter().filter(|record| selection.matches_scope(record.scope)).fold(size_of::<Vec<RecordMarker>>(), |mut bytes, record| {
        bytes = bytes.saturating_add(size_of::<RecordMarker>());
        let scope = record.scope;
        let record = record.entry;
        for (owned, inline) in [
            (record.timestamp.memory_size(), size_of_val(&record.timestamp)),
            (record.record_type.memory_size(), size_of_val(&record.record_type)),
            (scope.map_or(0, MemorySize::memory_size), size_of::<RecordScope>()),
        ] {
            if owned == usize::MAX {
                return usize::MAX;
            }
            bytes = bytes.saturating_add(owned.saturating_sub(inline));
        }
        bytes.saturating_add(record.message.as_ref().map_or_else(|| record.record_type.as_str().len(), String::len))
    })
}

pub(crate) fn scope_matches(scope: Option<&RecordScope>, selection: &DocumentSelection) -> bool {
    selection.matches_scope(scope)
}

/* === Private Functions === */

/* === Tests === */

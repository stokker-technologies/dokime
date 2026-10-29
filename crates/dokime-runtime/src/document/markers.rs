/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::document::{
    DocumentFailure,
    input::{DocumentSelection, Records},
};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::records::evidence::RecordMarker;

pub fn record_markers_for_selection(records: &[RecordEntry], selection: &DocumentSelection) -> crate::Result<Vec<RecordMarker>> {
    selected_markers(Records::Native(records.into()), selection)
}

pub(super) fn selected_markers(records: Records<'_>, selection: &DocumentSelection) -> crate::Result<Vec<RecordMarker>> {
    let count = records.iter().filter(|record| selection.matches_scope(record.scope)).count();
    let mut markers = Vec::new();
    markers.try_reserve_exact(count).map_err(|_| DocumentFailure::Limit)?;
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
pub(crate) fn scope_matches(scope: Option<&RecordScope>, selection: &DocumentSelection) -> bool {
    selection.matches_scope(scope)
}

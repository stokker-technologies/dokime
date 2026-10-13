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
use dokime_app::runtime::query::execution_history::ExecutionCheckpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn checkpoint_heap(source: &ExecutionCheckpoint) -> usize {
    let mut bytes = source
        .execution_id
        .as_str()
        .len()
        .saturating_add(harmos::Message::encoded_len(&source.version));
    for cursor in [&source.position, &source.evidence_through] {
        bytes = bytes
            .saturating_add(cursor.history.store.as_str().len())
            .saturating_add(cursor.history.epoch.as_str().len());
    }
    bytes
}

/* === Private Functions === */

/* === Tests === */

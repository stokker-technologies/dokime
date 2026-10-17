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
use crate::runtime_state::RuntimeState;
use crate::traceability::TraceabilitySnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn traceability_snapshots(state: &RuntimeState) -> Vec<TraceabilitySnapshot> {
    crate::traceability::snapshots_from_state(state, None, crate::model::time::now_utc())
}

/* === Private Functions === */

/* === Tests === */

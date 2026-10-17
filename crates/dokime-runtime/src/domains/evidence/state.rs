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
use crate::core::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn traceability_snapshots(state: &RuntimeState) -> Vec<TraceabilitySnapshot> {
    crate::traceability::snapshots_from_state(state, None, Timestamp::now_utc())
}

/* === Private Functions === */

/* === Tests === */

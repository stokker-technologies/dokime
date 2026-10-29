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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl NetworkCollector {
    // Network List Manager Connectivity: Internet Over IPv4 Or IPv6, Without Own Traffic.
    pub(super) fn internet() -> Option<bool> {
        crate::platform::windows::connectivity::internet()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

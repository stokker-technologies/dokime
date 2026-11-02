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
    // NetworkManager's Last Connectivity Verdict, Without Own Traffic.
    pub(super) fn internet(&self) -> Option<bool> {
        crate::platform::unix::connectivity::internet(std::time::Duration::from_millis(self.options.timing.network_probe_timeout_ms))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

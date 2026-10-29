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

impl BluetoothCollector {
    pub(super) fn bluetooth_values(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
        crate::platform::unix::bluetooth::linux_bluetooth_snapshot(options, timeout)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

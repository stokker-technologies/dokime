/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::*;

/* === Private Functions === */

impl BluetoothCollector {
    pub(super) fn bluetooth_values(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
        crate::platform::unix::bluetooth::linux_bluetooth_snapshot(options, timeout)
    }
}

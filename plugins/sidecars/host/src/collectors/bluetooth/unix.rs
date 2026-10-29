/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

impl BluetoothCollector {
    pub(super) fn bluetooth_values(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
        crate::platform::unix::bluetooth::linux_bluetooth_snapshot(options, timeout)
    }
}

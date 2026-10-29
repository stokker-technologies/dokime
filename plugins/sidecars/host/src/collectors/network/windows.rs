/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

impl NetworkCollector {
    // Network List Manager Connectivity: Internet Over IPv4 Or IPv6, Without Own Traffic.
    pub(super) fn internet() -> Option<bool> {
        crate::platform::windows::connectivity::internet()
    }
}

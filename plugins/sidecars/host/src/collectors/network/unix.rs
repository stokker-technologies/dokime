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
    // NetworkManager's Last Connectivity Verdict, Without Own Traffic.
    pub(super) fn internet() -> Option<bool> {
        crate::platform::unix::connectivity::internet(crate::host::network::POLICY.attempt_timeout)
    }
}

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

/* === Implementations === */

impl SystemCollector {
    pub(super) fn power_profile_context() -> PowerProfileContext {
        PowerProfileContext::default()
    }
}

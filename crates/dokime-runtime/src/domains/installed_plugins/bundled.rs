/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::InstalledPlugin;

pub(crate) fn builtin_plugins() -> Vec<InstalledPlugin> {
    vec![dokime_host::installed_plugin()]
}

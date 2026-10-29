/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

use obc_plugin::ObcPlugin;

fn main() -> std::io::Result<()> {
    dokime::plugin::run::<ObcPlugin>()
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use obc_plugin::ObcPlugin;

fn main() -> std::io::Result<()> {
    dokime::plugin::run::<ObcPlugin>()
}

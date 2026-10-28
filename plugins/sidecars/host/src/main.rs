/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Public Functions === */

fn main() -> std::io::Result<()> {
    dokime::plugin::run::<dokime_host::HostPlugin>()
}

/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


fn main() -> std::io::Result<()> {
    dokime::plugin::run::<{{crate}}::Plugin>()
}

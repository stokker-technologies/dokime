/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


struct Unnamed;

struct Capitalized;

struct Unscoped;


#[sdk::template(label = "Unnamed")]
impl Unnamed {}

#[sdk::template(id = "checks.Probe", label = "Capitalized")]
impl Capitalized {}

#[sdk::template(id = "probe", label = "Unscoped")]
impl Unscoped {}


fn main() {}

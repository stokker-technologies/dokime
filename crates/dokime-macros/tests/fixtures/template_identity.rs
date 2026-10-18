/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Unnamed;

struct Capitalized;

struct Unscoped;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[sdk::template(label = "Unnamed")]
impl Unnamed {}

#[sdk::template(id = "checks.Probe", label = "Capitalized")]
impl Capitalized {}

#[sdk::template(id = "probe", label = "Unscoped")]
impl Unscoped {}

/* === Public Functions === */

/* === Private Functions === */
fn main() {}

/* === Tests === */

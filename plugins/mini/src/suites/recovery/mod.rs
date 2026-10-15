/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod power;

/* === Re-Exports === */
pub use power::{PowerLoss, Samples};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
pub const SUITE: &str = "mini.charger.recovery";
pub const TEMPLATE: &str = "mini.charger.recovery.power";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Suite;

/* === Enums === */
#[dokime::catalog]
pub enum Templates {
    PowerLoss(power::PowerLoss),
}
/* === Traits === */

/* === Implementations === */
#[dokime::suite(id = SUITE, label = "Power-Loss Recovery", tags = ["device", "recovery"], templates = Templates, description = "Lose Charger power, clear the latched fault, and verify a controlled restart.")]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

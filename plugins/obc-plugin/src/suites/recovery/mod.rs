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
pub const SUITE: &str = "obc.charger.recovery";
pub const TEMPLATE: &str = "obc.charger.recovery.power";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Suite;

/* === Enums === */
#[dokime_runtime::catalog]
pub enum Templates {
    PowerLoss(power::PowerLoss),
}
/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(id = SUITE, label = "Power-Loss Recovery", tags = ["device", "recovery"], templates = Templates, description = "Lose Charger power, clear the latched fault, and verify a controlled restart.")]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

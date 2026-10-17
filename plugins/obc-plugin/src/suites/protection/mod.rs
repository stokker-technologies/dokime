/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod overcurrent;

/* === Re-Exports === */
pub use overcurrent::{Overcurrent, Samples};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
pub const SUITE: &str = "obc.charger.protection";
pub const TEMPLATE: &str = "obc.charger.protection.overcurrent";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Suite;

/* === Enums === */
#[dokime::catalog]
pub enum Templates {
    Overcurrent(overcurrent::Overcurrent),
}
/* === Traits === */

/* === Implementations === */
#[dokime::suite(id = SUITE, label = "Overcurrent Protection", tags = ["device", "protection"], templates = Templates, description = "Increase the load and verify that the RMS protection latch disconnects the Charger.")]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

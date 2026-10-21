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

#[dokime::suite(id = SUITE, label = "Charger Protection", tags = ["device", "protection"], templates = Templates, description = "Verify that the Charger's protection disconnects it once the load draws too much current.")]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

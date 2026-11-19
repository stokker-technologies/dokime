/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Transient Suite Definition

/* === Modules === */

pub mod overcurrent;
pub mod overvoltage;
pub mod startup;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::Suite;

/* === Constants === */

pub const SUITE: &str = "obc.transient";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn suite() -> Suite {
    Suite::of::<()>().identifier(SUITE).label("Transient").template(startup::template()).template(overcurrent::template()).template(overvoltage::template())
}

/* === Private Functions === */

/* === Tests === */

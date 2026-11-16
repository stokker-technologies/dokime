/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Steady-State Suite Definition

/* === Modules === */

pub mod single_phase;
pub mod three_phase;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::Suite;

/* === Constants === */

pub const SUITE: &str = "obc.steady_state";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn suite() -> Suite {
    Suite::new(SUITE, "Steady State").template(single_phase::template()).template(three_phase::template())
}

/* === Private Functions === */

/* === Tests === */

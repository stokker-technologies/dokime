/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Independent Benchmark Workloads

/* === Modules === */

pub mod exact_values;
pub mod observers;
pub mod plots;
pub mod streams;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::Suite;

/* === Constants === */

pub const SUITE: &str = "obc.benchmark";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn suite() -> Suite {
    Suite::new(SUITE, "Benchmark")
        .template(streams::template())
        .template(plots::template())
        .template(observers::template())
        .template(exact_values::template())
}

/* === Private Functions === */

/* === Tests === */

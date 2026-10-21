/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Benchmark-Only High-Volume Evidence And Prompt Fixture

/* === Modules === */
mod capture;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Suite;

/* === Enums === */
#[dokime_runtime::catalog]
pub enum Templates {
    Capture(capture::Capture),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(
    id = "obc.benchmarks", label = "Benchmark Fixtures", templates = Templates,
    description = "Generated high-volume evidence for system benchmarks; excluded from product builds."
)]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

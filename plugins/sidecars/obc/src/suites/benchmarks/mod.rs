/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Benchmark-Only High-Volume Evidence and Prompt Fixture

/* === Modules === */
mod capture;
mod ingest;

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
#[dokime::catalog]
pub enum Templates {
    Capture(capture::Capture),
    ScalarZero(ingest::ScalarZero),
    ScalarOne(ingest::ScalarOne),
    ScalarMany(ingest::ScalarMany),
    WideZero(ingest::WideZero),
    WideOne(ingest::WideOne),
    WideMany(ingest::WideMany),
}

/* === Traits === */

/* === Implementations === */
#[dokime::suite(
    id = "obc.benchmarks", label = "Benchmark Fixtures", templates = Templates,
    description = "Generated high-volume evidence for system benchmarks; excluded from product builds."
)]
impl Suite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

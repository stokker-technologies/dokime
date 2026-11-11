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
mod payload_4k_zero;
mod payload_64k_zero;
mod scalar_many;
mod scalar_one;
mod scalar_zero;
mod wide_many;
mod wide_one;
mod wide_zero;

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
    Capture(capture::Template),
    ScalarZero(scalar_zero::Template),
    ScalarOne(scalar_one::Template),
    ScalarMany(scalar_many::Template),
    WideZero(wide_zero::Template),
    WideOne(wide_one::Template),
    WideMany(wide_many::Template),
    Payload4kZero(payload_4k_zero::Template),
    Payload64kZero(payload_64k_zero::Template),
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

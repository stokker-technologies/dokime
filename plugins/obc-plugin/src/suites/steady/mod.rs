/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod measurement;

/* === Re-Exports === */

pub use measurement::{BenchEvent, Samples, Steady};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const STEADY_SUITE: &str = "obc.charger.steady";
pub const STEADY_TEMPLATE: &str = "obc.charger.steady.measurement";
const STEADY_TEMPLATE_ID: &str = "0197f000-0000-7000-8000-000000000002";
pub const STEADY_ACCEPTANCE_CRITERION: &str = "obc.charger.steady.acceptance";
pub const STEADY_ACCEPTANCE_CRITERION_LABEL: &str = "Charging Input Measurement";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SteadySuite;

/* === Enums === */

#[dokime_runtime::catalog]
pub enum Templates {
    Steady(measurement::Steady),
}

/* === Traits === */

/* === Implementations === */

#[dokime_runtime::suite(
    id = STEADY_SUITE,
    label = "Steady Charging",
    tags = ["device", "steady", "ac"],
    templates = Templates, description = "Measure AC voltage and load sweeps with an execution-owned Charger and SDK sampler.")]
impl SteadySuite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

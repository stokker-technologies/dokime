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
pub const STEADY_ACCEPTANCE_CRITERION_LABEL: &str = "Steady-state trace and operator acceptance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SteadySuite;

/* === Enums === */

#[dokime::catalog]
pub enum Templates {
    Steady(measurement::Steady),
}

/* === Traits === */

/* === Implementations === */

#[dokime::suite(
    id = STEADY_SUITE,
    label = "Charger Steady State",
    tags = ["device", "steady", "ac"],
    templates = Templates, description = "Measure the Charger's steady-state input across AC voltage and load sweeps.")]
impl SteadySuite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

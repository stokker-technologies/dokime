/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod measurement;

pub use measurement::{BenchEvent, Samples, Steady};

pub const STEADY_SUITE: &str = "obc.charger.steady";
pub const STEADY_TEMPLATE: &str = "obc.charger.steady.measurement";
const STEADY_TEMPLATE_ID: &str = "0197f000-0000-7000-8000-000000000002";
pub const STEADY_ACCEPTANCE_CRITERION: &str = "obc.charger.steady.acceptance";
pub const STEADY_ACCEPTANCE_CRITERION_LABEL: &str = "Steady-state trace and operator acceptance";

pub struct SteadySuite;

#[dokime::catalog]
pub enum Templates {
    Steady(measurement::Steady),
}

#[dokime::suite(
    id = STEADY_SUITE,
    label = "Charger Steady State",
    tags = ["device", "steady", "ac"],
    templates = Templates, description = "Measure the Charger's steady-state input across AC voltage and load sweeps.")]
impl SteadySuite {}

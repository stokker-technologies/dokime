/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Asked before the startup sequence goes on. The charger is energized while
// the operator looks, so the question is raised to Warning.
#[dokime::prompt(
    id = "obc.charger.startup.check",
    label = "Inspect Powered Charger",
    submit = "Submit",
    level = warning,
    description = "Check that the charger status indicator is steady and no fault is shown, then record your observation."
)]
pub struct StartupCheck {
    #[prompt(label = "Operator Note", default = "")]
    pub operator_note: String,

    #[prompt(label = "Charger Inspection Passed", default = false)]
    pub accepted: bool,
}

// Asked before the steady-state measurement decides, with the charger under
// load, so the question is raised to Warning.
#[dokime::prompt(
    id = "obc.charger.steady.check",
    label = "Inspect Charging State",
    submit = "Submit",
    level = warning,
    description = "Check that the charger status indicator is steady and no fault is shown, then record your observation."
)]
pub struct SteadyCheck {
    #[prompt(label = "Operator Note", default = "")]
    pub operator_note: String,

    #[prompt(label = "Charger Inspection Passed", default = false)]
    pub accepted: bool,
}

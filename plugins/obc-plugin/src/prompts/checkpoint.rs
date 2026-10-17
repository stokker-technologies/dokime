/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Asked before the startup sequence goes on. The charger is energized while
// the operator looks, so the question is raised to Warning.
#[dokime_runtime::prompt(
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
#[dokime_runtime::prompt(
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

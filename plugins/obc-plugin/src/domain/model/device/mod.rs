/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manual device observation

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

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct DeviceState {
    #[row(description = "Whether the manual demonstration device is powered.")]
    pub enabled: bool,

    #[row(description = "Observed RMS voltage in volts; zero while off or faulted.")]
    pub voltage_rms_v: f64,

    #[row(description = "Latched fault, when present: overcurrent or power_loss.")]
    pub fault: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

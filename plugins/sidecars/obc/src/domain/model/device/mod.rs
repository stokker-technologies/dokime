/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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

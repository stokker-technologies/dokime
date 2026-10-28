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
use crate::domain::model::device::DeviceState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(
    id = "obc.device.read",
    visibility = Public,
    response = DeviceState,
    description = "Reads the manual simulated device without changing its power or advancing a charger execution.",
    policy = super::POLICY,
    resources = [crate::Resources],
    grpc(package = "obc.device.v1", service = "Device", method = "Read")
)]
#[derive(Clone, PartialEq)]
pub struct ReadDevice {}

#[dokime::operation(
    id = "obc.device.set_power",
    visibility = Public,
    response = DeviceState,
    description = "Powers the manual simulated device on or off, then reads its current state. Refuses a stopped bench or powering on a faulted device.",
    policy = super::POLICY,
    resources = [crate::Resources],
    grpc(package = "obc.device.v1", service = "Device", method = "SetPower")
)]
#[derive(Clone, PartialEq)]
pub struct SetPower {
    pub enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

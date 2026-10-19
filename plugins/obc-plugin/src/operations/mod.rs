/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod connections;
pub mod device;
pub mod readiness;
pub mod status;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime_runtime::catalog]
pub enum Implemented {
    DeviceReader(device::DeviceReader),
    PowerControl(device::PowerControl),
    StatusProbe(status::StatusProbe),
    ReadinessProbe(readiness::ReadinessProbe),
    ConnectionListing(connections::ConnectionListing),
}

#[dokime_runtime::catalog]
pub enum Operations {
    #[nested]
    Bench(crate::domain::operations::Operations),

    #[nested]
    Analyses(crate::analyses::Implemented),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

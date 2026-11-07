/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod chamber;
pub mod soak;
pub mod status;

/* === Re-Exports === */

pub use chamber::{DetectPort, OpenHeater, ReadChamber, ReleaseChamber, SetTarget};
pub use soak::EncodeSummary;
pub use status::GetStatus;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    GetStatus(GetStatus),
    ReadChamber(ReadChamber),
    DetectPort(DetectPort),
    OpenHeater(OpenHeater),
    SetTarget(SetTarget),
    ReleaseChamber(ReleaseChamber),
    EncodeSummary(EncodeSummary),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

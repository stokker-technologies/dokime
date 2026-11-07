/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod charger;
pub mod connections;
pub mod device;
pub mod readiness;
pub mod status;
pub mod system;
pub mod wait;

/* === Re-Exports === */

pub use connections::ListConnections;
pub use device::{ReadDevice, SetPower};
pub use readiness::AskReadiness;
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
    ReadDevice(ReadDevice),
    SetPower(SetPower),
    GetStatus(GetStatus),
    AskReadiness(AskReadiness),
    ListConnections(ListConnections),
    CollectSystem(system::CollectSystem),
    Wait(wait::Wait),
    InitializeCharger(charger::InitializeCharger),
    StartCharger(charger::StartCharger),
    ReadCharger(charger::ReadCharger),
    SampleCharger(charger::SampleCharger),
    SampleChargerBatch(charger::SampleChargerBatch),
    SetChargerLoad(charger::SetChargerLoad),
    InjectChargerFault(charger::InjectChargerFault),
    ClearChargerFault(charger::ClearChargerFault),
    ProbeChargerRestart(charger::ProbeChargerRestart),
    StopCharger(charger::StopCharger),
    ReleaseCharger(charger::ReleaseCharger),
    EncodeChargerSummary(charger::EncodeChargerSummary),

    #[nested]
    Analyses(crate::analyses::Analyses),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

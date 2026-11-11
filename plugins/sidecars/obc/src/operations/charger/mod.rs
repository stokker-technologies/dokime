/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Operations

/* === Modules === */

pub mod clear_fault;
pub mod encode_summary;
pub mod initialize;
pub mod inject_fault;
pub mod probe_restart;
pub mod read;
pub mod release;
pub mod sample;
pub mod sample_batch;
pub mod set_load;
pub mod start;
pub mod stop;

/* === Re-Exports === */

pub use clear_fault::ClearChargerFault;
pub use encode_summary::EncodeChargerSummary;
pub use initialize::InitializeCharger;
pub use inject_fault::InjectChargerFault;
pub use probe_restart::ProbeChargerRestart;
pub use read::ReadCharger;
pub use release::ReleaseCharger;
pub use sample::SampleCharger;
pub use sample_batch::SampleChargerBatch;
pub use set_load::SetChargerLoad;
pub use start::StartCharger;
pub use stop::StopCharger;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

// Third Party Imports
use obc_runtime::device::{Device, Fault as DriverFault, Power as DriverPower};

// Local Imports
use crate::{
    Error,
    domain::model::charger::{Charger, Fault, Power, Supply},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct Chargers {
    next_id: AtomicU64,
    devices: Mutex<BTreeMap<u64, Device>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Chargers {
    fn with<T>(&self, device_id: u64, action: impl FnOnce(&mut Device) -> Result<T, Error>) -> Result<T, Error> {
        let mut devices = self.devices.lock().map_err(|_| Error::DeviceResourcePoisoned)?;
        let device = devices.get_mut(&device_id).ok_or_else(|| Error::Observation(format!("charger {device_id} is not open")))?;
        action(device)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn snapshot(device_id: u64, device: &Device) -> Charger {
    Charger {
        device_id,
        supply: Supply {
            voltage_rms_v: device.supply.voltage_rms_v,
            load_ohm: device.supply.load_ohm,
            frequency_hz: device.supply.frequency_hz,
            phase_deg: device.supply.phase_deg,
            current_limit_a: device.supply.current_limit_a,
        },
        power: match device.power {
            DriverPower::Off => Power::Off,
            DriverPower::Running => Power::Running,
            DriverPower::Faulted => Power::Faulted,
        },
        fault: device.fault.map(|fault| match fault {
            DriverFault::Overcurrent => Fault::Overcurrent,
            DriverFault::PowerLoss => Fault::PowerLoss,
        }),
        log: device.log.as_ref().map(|path| path.to_string_lossy().into_owned()),
    }
}

/* === Tests === */

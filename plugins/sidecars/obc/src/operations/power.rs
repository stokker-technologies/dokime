/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manual Control of One Demonstration Charger, Outside Any Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::device::{Charger, Power, Supply};
use dokime::operations::{Context, Operation};
use dokime::{Message, Result, Visibility};

/* === Constants === */

pub const READ: &str = "obc.device.read";
pub const SET_POWER: &str = "obc.device.set_power";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Read the Demonstration Charger Without Changing It.
#[derive(Message)]
#[dokime(package = "obc")]
pub struct DeviceRead {}

/// Energize or De-Energize the Demonstration Charger.
#[derive(Message)]
#[dokime(package = "obc")]
pub struct DeviceSetPower {
    pub enabled: bool,
}

/// The Demonstration Charger as Observed After the Call.
#[derive(Message)]
#[dokime(package = "obc")]
pub struct DeviceState {
    pub enabled: bool,
    pub voltage_rms_v: f64,
    pub fault: String,
}

/// The Process-Wide Demonstration Charger; Executions Own Separate Chargers.
pub struct Manual {
    charger: Arc<Mutex<Charger>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Manual {
    pub fn new() -> Result<Self> {
        Ok(Self {
            charger: Arc::new(Mutex::new(Charger::new(Supply::default())?)),
        })
    }

    pub fn operations(&self) -> [Operation; 2] {
        let charger = self.charger.clone();
        let read = Operation::new(READ, Visibility::Public, async move |_: &Context, _: DeviceRead| Ok(state(&charger)))
            .description("Report Whether the Demonstration Charger Is Energized, Its RMS Voltage and Any Latched Fault.");
        let charger = self.charger.clone();
        let set_power = Operation::new(SET_POWER, Visibility::Public, async move |_: &Context, request: DeviceSetPower| {
            {
                let mut device = charger.lock().unwrap_or_else(PoisonError::into_inner);
                if request.enabled {
                    device.start(0.0)?;
                } else {
                    device.stop();
                }
            }
            Ok(state(&charger))
        })
        .description("Energize or De-Energize the Demonstration Charger and Report Its Observed State.");
        [read, set_power]
    }

}

impl Drop for Manual {
    fn drop(&mut self) {
        self.charger.lock().unwrap_or_else(PoisonError::into_inner).stop();
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn state(charger: &Mutex<Charger>) -> DeviceState {
    let device = charger.lock().unwrap_or_else(PoisonError::into_inner);
    DeviceState {
        enabled: device.power == Power::Running,
        voltage_rms_v: device.reading(1.0).voltage_rms_v,
        fault: device.fault.map(|fault| fault.name().to_owned()).unwrap_or_default(),
    }
}

/* === Tests === */

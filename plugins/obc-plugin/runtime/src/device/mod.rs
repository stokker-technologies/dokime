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
use std::io::Write;
use std::path::{Path, PathBuf};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Supply {
    pub voltage_rms_v: f64,
    pub load_ohm: f64,
    pub frequency_hz: f64,
    pub phase_deg: f64,
    pub current_limit_a: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub elapsed_s: f64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub voltage_rms_v: f64,
    pub current_rms_a: f64,
    pub frequency_hz: f64,
    pub phase_deg: f64,
    pub input_power_w: f64,
    pub dc_voltage_v: f64,
    pub dc_current_a: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub supply: Supply,
    pub power: Power,
    pub fault: Option<Fault>,

    // The file the charger appends one status line to per sample, the way a
    // real unit writes its own log.
    #[serde(skip)]
    pub log: Option<PathBuf>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Power {
    #[default]
    Off,

    Running,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fault {
    Overcurrent,
    PowerLoss,
}

/* === Traits === */

/* === Implementations === */

impl Default for Supply {
    fn default() -> Self {
        Self { voltage_rms_v: 230.0, load_ohm: 100.0, frequency_hz: 50.0, phase_deg: 15.0, current_limit_a: 5.0 }
    }
}

impl Device {
    pub fn initialize(supply: Supply) -> Result<Self, Error> {
        if !supply.voltage_rms_v.is_finite()
            || !(0.0..=300.0).contains(&supply.voltage_rms_v)
            || !supply.load_ohm.is_finite()
            || supply.load_ohm <= 0.0
            || !supply.frequency_hz.is_finite()
            || !(1.0..=1000.0).contains(&supply.frequency_hz)
            || !supply.phase_deg.is_finite()
            || supply.phase_deg.abs() >= 90.0
            || !supply.current_limit_a.is_finite()
            || supply.current_limit_a <= 0.0
        {
            return Err(Error::InvalidSupply);
        }
        Ok(Self { supply, ..Default::default() })
    }

    // Logs every sample to this file from now on.
    pub fn logging(&mut self, path: impl AsRef<Path>) {
        self.log = Some(path.as_ref().to_owned());
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.fault.is_some() {
            return Err(Error::FaultLatched);
        }
        self.power = Power::Running;
        // A powered unit has its log open, so a watch started now takes it as its baseline.
        let opened = self.log.as_ref().map(|log| std::fs::OpenOptions::new().create(true).append(true).open(log));
        if let Some(Err(error)) = opened {
            tracing::warn!(%error, "the charger log was not opened");
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.fault.is_none() {
            self.power = Power::Off;
        }
    }

    pub fn inject(&mut self, fault: Fault) {
        self.fault = Some(fault);
        self.power = Power::Faulted;
    }

    pub fn clear(&mut self) {
        self.power = Power::Off;
        self.fault = None;
    }

    pub fn sample(&mut self, elapsed_s: f64) -> Reading {
        let envelope = 1.0 + 0.5 * (-elapsed_s.max(0.0) / 0.02).exp();
        let current_rms = self.supply.voltage_rms_v / self.supply.load_ohm * envelope;
        if self.power == Power::Running && current_rms > self.supply.current_limit_a {
            self.inject(Fault::Overcurrent);
        }
        let powered = self.power == Power::Running;
        let voltage_rms = if powered { self.supply.voltage_rms_v } else { 0.0 };
        let current_rms = if powered { current_rms } else { 0.0 };
        let angle = std::f64::consts::TAU * self.supply.frequency_hz * elapsed_s;
        let input_power_w = voltage_rms * current_rms * self.supply.phase_deg.to_radians().cos();
        let dc_voltage_v = if powered { 400.0 } else { 0.0 };
        let dc_current_a = if powered { input_power_w * 0.94 / dc_voltage_v } else { 0.0 };

        if let Some(log) = &self.log {
            // A log the charger cannot write loses its lines, never the sample.
            let line = format!("t={elapsed_s:.3} state={:?} input_power_w={input_power_w:.1}\n", self.power);
            let written =
                std::fs::OpenOptions::new().create(true).append(true).open(log).and_then(|mut file| file.write_all(line.as_bytes()));
            if let Err(error) = written {
                tracing::warn!(%error, log = %log.display(), "the charger log was not written");
            }
        }

        Reading {
            elapsed_s,
            voltage_v: std::f64::consts::SQRT_2 * voltage_rms * angle.sin(),
            current_a: std::f64::consts::SQRT_2 * current_rms * (angle - self.supply.phase_deg.to_radians()).sin(),
            voltage_rms_v: voltage_rms,
            current_rms_a: current_rms,
            frequency_hz: self.supply.frequency_hz,
            phase_deg: self.supply.phase_deg,
            input_power_w,
            dc_voltage_v,
            dc_current_a,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

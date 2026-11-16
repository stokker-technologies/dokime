/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Simulated Onboard Charger on Its AC Testbench

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::f64::consts::{SQRT_2, TAU};

// Third Party Imports

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

// Battery Voltage While the Charger Is Running.
const DC_VOLTAGE_V: f64 = 400.0;

// Conversion Efficiency From AC Input to DC Output.
const EFFICIENCY: f64 = 0.94;

// Time Constant of the Start-Up Inrush Envelope.
const INRUSH_S: f64 = 0.02;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The AC Supply and Load the Bench Applies to the Charger.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Supply {
    pub voltage_rms_v: f64,
    pub load_ohm: f64,
    pub reactance_ohm: f64,
    pub phases: u32,
    pub voltage_limit_v: f64,
    pub frequency_hz: f64,
    pub current_limit_a: f64,
}

/// One Instantaneous Reading of the Charger's Input and Output.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// A Deterministic Charger: Inrush, Phase Lag, Overcurrent and Overvoltage Latches.
#[derive(Clone, Debug, PartialEq)]
pub struct Charger {
    pub supply: Supply,
    pub power: Power,
    pub fault: Option<Fault>,
    energized_s: f64,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Power {
    #[default]
    Off,
    Running,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Overcurrent,
    Overvoltage,
}

/* === Traits === */

/* === Implementations === */

impl Default for Supply {
    fn default() -> Self {
        Self {
            voltage_rms_v: 230.0,
            load_ohm: 100.0,
            reactance_ohm: 0.0,
            phases: 1,
            voltage_limit_v: 270.0,
            frequency_hz: 50.0,
            current_limit_a: 5.0,
        }
    }
}

impl Supply {
    pub fn validate(self) -> Result<Self> {
        let voltage_valid = (0.0..=300.0).contains(&self.voltage_rms_v);
        let impedance_valid = self.load_ohm.is_finite() && self.load_ohm > 0.0 && self.reactance_ohm.is_finite();
        let phases_valid = matches!(self.phases, 1 | 3);
        let frequency_valid = (1.0..=1000.0).contains(&self.frequency_hz);
        let protection_valid = self.voltage_limit_v.is_finite()
            && self.voltage_limit_v > 0.0
            && self.current_limit_a.is_finite()
            && self.current_limit_a > 0.0;
        if !(voltage_valid && impedance_valid && phases_valid && frequency_valid && protection_valid) {
            return Err(Error::Invalid("Initialize Charger: Choose 0–300 V, Positive Resistance, Finite Reactance, One or Three Phases, 1–1000 Hz and Positive Protection Limits".into()));
        }
        Ok(self)
    }
}

impl Fault {
    pub fn name(self) -> &'static str {
        match self {
            Self::Overcurrent => "overcurrent",
            Self::Overvoltage => "overvoltage",
        }
    }
}

impl Charger {
    pub fn new(supply: Supply) -> Result<Self> {
        Ok(Self {
            supply: supply.validate()?,
            power: Power::Off,
            fault: None,
            energized_s: 0.0,
        })
    }

    /// Energize the Supply; a Latched Fault Refuses the Restart.
    pub fn start(&mut self, at_s: f64) -> Result<()> {
        if let Some(fault) = self.fault {
            return Err(Error::Invalid(format!(
                "Start Charger: Restore a Safe Supply and Initialize a New Charger After the Latched {} Fault",
                fault.name()
            )));
        }
        self.power = Power::Running;
        self.energized_s = at_s;
        Ok(())
    }

    /// De-Energize the Supply; a Latched Fault Remains Set.
    pub fn stop(&mut self) {
        if self.fault.is_none() {
            self.power = Power::Off;
        }
    }

    /// Apply a New Load; Protection Evaluates the Resulting Current at Once.
    pub fn set_load(&mut self, load_ohm: f64, at_s: f64) -> Result<()> {
        self.supply = Supply { load_ohm, ..self.supply }.validate()?;
        self.protect(at_s);
        Ok(())
    }

    /// Apply a New Supply Voltage and Evaluate Protection Immediately.
    pub fn set_voltage(&mut self, voltage_rms_v: f64, at_s: f64) -> Result<()> {
        self.supply = Supply { voltage_rms_v, ..self.supply }.validate()?;
        self.protect(at_s);
        Ok(())
    }

    fn inject(&mut self, fault: Fault) {
        self.fault = Some(fault);
        self.power = Power::Faulted;
    }

    /// Sample the Charger, Latching Protection First.
    pub fn sample(&mut self, elapsed_s: f64) -> Reading {
        self.protect(elapsed_s);
        self.reading(elapsed_s)
    }

    /// Read Without Changing the Charger.
    pub fn reading(&self, elapsed_s: f64) -> Reading {
        let powered = self.power == Power::Running;
        let voltage_rms_v = if powered { self.supply.voltage_rms_v } else { 0.0 };
        let current_rms_a = if powered { self.drawn_a(elapsed_s) } else { 0.0 };
        let angle = TAU * self.supply.frequency_hz * elapsed_s;
        let lag = self.supply.reactance_ohm.atan2(self.supply.load_ohm);
        let input_power_w = f64::from(self.supply.phases) * voltage_rms_v * current_rms_a * lag.cos();
        let dc_voltage_v = if powered { DC_VOLTAGE_V } else { 0.0 };
        Reading {
            elapsed_s,
            voltage_v: SQRT_2 * voltage_rms_v * angle.sin(),
            current_a: SQRT_2 * current_rms_a * (angle - lag).sin(),
            voltage_rms_v,
            current_rms_a,
            frequency_hz: self.supply.frequency_hz,
            phase_deg: lag.to_degrees(),
            input_power_w,
            dc_voltage_v,
            dc_current_a: if powered { input_power_w * EFFICIENCY / DC_VOLTAGE_V } else { 0.0 },
        }
    }

    fn drawn_a(&self, elapsed_s: f64) -> f64 {
        let since = (elapsed_s - self.energized_s).max(0.0);
        self.supply.voltage_rms_v / self.supply.load_ohm.hypot(self.supply.reactance_ohm) * (1.0 + 0.5 * (-since / INRUSH_S).exp())
    }

    fn protect(&mut self, at_s: f64) {
        if self.power != Power::Running {
            return;
        }
        if self.supply.voltage_rms_v > self.supply.voltage_limit_v {
            self.inject(Fault::Overvoltage);
        } else if self.drawn_a(at_s) > self.supply.current_limit_a {
            self.inject(Fault::Overcurrent);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

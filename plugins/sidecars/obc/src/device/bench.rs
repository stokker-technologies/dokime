/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution-Owned Charger, Synchronization and Model Clock

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::{Charger, Fault, Power, Reading, Supply};
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Execution's Charger and Model Clock; No Lock or Clock State Escapes.
pub struct Bench {
    state: Mutex<State>,
}
struct State {
    charger: Charger,
    index: u64,
    step_s: f64,
    interval: Duration,
    label: &'static str,
    sequence: Option<(u64, u64)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bench {
    pub fn new(supply: Supply, step_s: Option<f64>) -> Result<Self> {
        let interval = Duration::from_millis(1);
        let step_s = step_s.unwrap_or(interval.as_secs_f64());
        if !step_s.is_finite() || step_s <= 0.0 {
            return Err(Error::Invalid("Open Bench: Choose a Positive Sample Interval".into()));
        }
        Ok(Self {
            state: Mutex::new(State {
                charger: Charger::new(supply)?,
                index: 0,
                step_s,

                interval,
                label: "idle",
                sequence: None,
            }),
        })
    }
    pub fn configure_interval(&self, interval: Duration, follow_host: bool) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.interval = interval;
        if follow_host {
            state.step_s = interval.as_secs_f64();
        }
    }
    pub fn interval(&self) -> Duration {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).interval
    }
    pub fn index(&self) -> u64 {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).index
    }
    pub fn step_s(&self) -> f64 {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).step_s
    }
    pub fn label(&self, label: &'static str) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).label = label;
    }
    pub fn sequence(&self, checkpoint: u64, total: u64) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).sequence = Some((checkpoint, total));
    }
    pub fn now_s(&self) -> f64 {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.index as f64 * state.step_s
    }
    pub fn time_s(&self, index: u64) -> f64 {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        index as f64 * state.step_s
    }
    pub fn charger(&self) -> Charger {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).charger.clone()
    }
    pub fn start(&self) -> Result<Supply> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let at_s = state.index as f64 * state.step_s;
        state.charger.start(at_s)?;
        state.label = "running";
        Ok(state.charger.supply)
    }
    pub fn stop(&self) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).charger.stop();
    }
    pub fn set_load(&self, load_ohm: f64) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let at_s = state.index as f64 * state.step_s;
        state.charger.set_load(load_ohm, at_s)
    }
    pub fn set_voltage(&self, voltage_v: f64) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let at_s = state.index as f64 * state.step_s;
        state.charger.set_voltage(voltage_v, at_s)
    }
    pub fn advance_by(&self, samples: u64) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).index += samples;
    }
    pub fn samples_for(&self, duration_s: f64) -> Result<u64> {
        let samples = (duration_s / self.step_s()).ceil();
        if !samples.is_finite() || !(1.0..=100_000.0).contains(&samples) {
            return Err(Error::Invalid("Size the Run: Choose a Duration Within 1 to 100000 Samples".into()));
        }
        Ok(samples as u64)
    }
    pub fn sample(&self) -> (u64, Reading, &'static str) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let index = state.index;
        let at_s = index as f64 * state.step_s;
        let reading = state.charger.sample(at_s);
        let phase = match state.sequence {
            _ if state.charger.power == Power::Faulted => "fault",
            Some(_) if index == 0 => "start",
            Some((_, total)) if index + 1 == total => "complete",
            Some((checkpoint, _)) if index + 1 == checkpoint => "checkpoint",
            Some((checkpoint, _)) if index < checkpoint => "before_checkpoint",
            Some(_) => "after_checkpoint",
            None => state.label,
        };
        state.index += 1;
        (index, reading, phase)
    }
    pub fn fault(&self) -> Option<Fault> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).charger.fault
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

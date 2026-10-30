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
use std::time::Duration;

// Third Party Imports

// Local Imports

/* === Constants === */

// Static Operation Deadlines; Runtime Settlement Must Fit The Cancellation Grace.
pub const HOLD_BUDGET: Duration = Duration::from_secs(1 << 32);
pub const CANCELLATION_GRACE: Duration = Duration::from_secs(60);

pub const DEFAULT: CpuLoadSettings = CpuLoadSettings {
    period_ms: 250,
    sample_ms: 250,
    slice_ms: 10,
    proportional_gain: 0.5,
    integral_gain: 0.5,
    integral_min: -1.0,
    integral_max: 1.0,
    settlement_ms: 2000,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::section(label = "CPU Load Control")]
#[derive(Clone, Debug, PartialEq)]
pub struct CpuLoadSettings {
    #[setting(label = "Controller Period [ms]", default = DEFAULT.period_ms, min = 1, max = 60_000)]
    pub period_ms: u32,

    #[setting(label = "CPU Sampling Period [ms]", default = DEFAULT.sample_ms, min = 1, max = 60_000)]
    pub sample_ms: u32,

    #[setting(label = "Worker Slice [ms]", default = DEFAULT.slice_ms, min = 1, max = 1000)]
    pub slice_ms: u32,

    #[setting(label = "Proportional Gain", default = DEFAULT.proportional_gain, min = 0.0)]
    pub proportional_gain: f64,

    #[setting(label = "Integral Gain [1/s]", default = DEFAULT.integral_gain, min = 0.0)]
    pub integral_gain: f64,

    #[setting(label = "Integrator Minimum", default = DEFAULT.integral_min)]
    pub integral_min: f64,

    #[setting(label = "Integrator Maximum", default = DEFAULT.integral_max)]
    pub integral_max: f64,

    #[setting(label = "Worker Settlement Budget [ms]", default = DEFAULT.settlement_ms, min = 1, max = 60_000)]
    pub settlement_ms: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for CpuLoadSettings {
    fn default() -> Self {
        DEFAULT
    }
}
impl CpuLoadSettings {
    pub fn validate(&self) -> crate::Result<()> {
        if self.period_ms == 0
            || self.sample_ms == 0
            || self.slice_ms == 0
            || self.settlement_ms < self.slice_ms
            || self.settlement() > CANCELLATION_GRACE
            || self.sample_ms > self.period_ms
            || self.slice_ms > self.period_ms
        {
            return Err(crate::Error::Probe(
                "CPU timing must be positive; sampling and slices must fit a control period; settlement must cover a slice and fit the cancellation grace".into(),
            ));
        }
        if !self.proportional_gain.is_finite()
            || self.proportional_gain < 0.0
            || !self.integral_gain.is_finite()
            || self.integral_gain < 0.0
            || !self.integral_min.is_finite()
            || !self.integral_max.is_finite()
            || self.integral_min > 0.0
            || self.integral_max < 0.0
            || self.integral_min >= self.integral_max
        {
            return Err(crate::Error::Probe("CPU gains must be finite and nonnegative; finite integrator bounds must bracket zero".into()));
        }
        Ok(())
    }
    pub fn period(&self) -> Duration {
        Duration::from_millis(u64::from(self.period_ms))
    }
    pub fn sample_period(&self) -> Duration {
        Duration::from_millis(u64::from(self.sample_ms))
    }
    pub fn slice(&self) -> Duration {
        Duration::from_millis(u64::from(self.slice_ms))
    }
    pub fn settlement(&self) -> Duration {
        Duration::from_millis(u64::from(self.settlement_ms))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_timing_and_gains() {
        assert!(DEFAULT.validate().is_ok());
        for settings in [
            CpuLoadSettings { period_ms: 0, ..DEFAULT },
            CpuLoadSettings {
                sample_ms: DEFAULT.period_ms + 1,
                ..DEFAULT
            },
            CpuLoadSettings { settlement_ms: 1, ..DEFAULT },
            CpuLoadSettings { settlement_ms: u32::MAX, ..DEFAULT },
            CpuLoadSettings { proportional_gain: f64::NAN, ..DEFAULT },
            CpuLoadSettings { integral_max: -1.0, ..DEFAULT },
        ] {
            assert!(settings.validate().is_err());
        }
    }
}

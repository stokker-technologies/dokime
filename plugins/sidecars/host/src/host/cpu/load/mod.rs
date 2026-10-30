/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod config;
#[cfg(feature = "runtime")]
mod controller;
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "runtime")]
mod workers;

/* === Re-Exports === */

pub use config::CpuLoadSettings;

/* === Imports === */

use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(1 << 32),
    attempt_timeout: std::time::Duration::from_secs(1 << 32),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "host.cpu.load.hold", response = LoadSummary, policy = POLICY, visibility = Public)]
#[derive(Clone, Debug, PartialEq)]
pub struct LoadHold {
    pub target: LoadTarget,
    pub threads: Option<u16>,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct LoadSummary {
    pub target: LoadTarget,
    pub mean: f64,
    pub error_rms: f64,
    pub duration_s: f64,
}

#[dokime::stream(
    id = "host.cpu.load.control",
    window = 4096,
    label = "CPU Load Control",
    description = "Measured closed-loop CPU load and generator throughput per controller period."
)]
#[derive(Clone, Debug, PartialEq)]
pub struct LoadControl {
    #[row(role = state, label = "Target Mode")]
    pub mode: String,

    #[row(label = "Target [%]", unit = "%")]
    pub target_percent: Option<f64>,

    #[row(label = "Measured [%]", unit = "%")]
    pub measured_percent: Option<f64>,

    #[row(label = "Target [FLOP/s]", unit = "FLOP/s")]
    pub target_flops: Option<f64>,

    #[row(label = "Measured [FLOP/s]", unit = "FLOP/s")]
    pub measured_flops: Option<f64>,

    #[row(label = "Duty [%]", unit = "%")]
    pub duty_percent: f64,

    #[row(label = "Generator [FLOP/s]", unit = "FLOP/s")]
    pub generator_flops: f64,
}

/* === Enums === */

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadTarget {
    Percent(f64),
    Flops(f64),
}

/* === Traits === */

/* === Implementations === */

impl LoadTarget {
    pub fn validate(self) -> crate::Result<()> {
        let valid = match self {
            Self::Percent(value) => value.is_finite() && value > 0.0 && value <= 100.0,
            Self::Flops(value) => value.is_finite() && value > 0.0,
        };
        if valid {
            Ok(())
        } else {
            Err(crate::Error::Probe("CPU target must be finite: percent in (0,100] or FLOP/s greater than zero".into()))
        }
    }
    pub fn value(self) -> f64 {
        match self {
            Self::Percent(value) | Self::Flops(value) => value,
        }
    }
    pub fn approximate_flops(self, capacity_gflops: f64) -> f64 {
        match self {
            Self::Percent(value) => capacity_gflops * 1_000_000_000.0 * value / 100.0,
            Self::Flops(value) => value,
        }
    }
    pub fn approximate_percent(self, capacity_gflops: f64) -> f64 {
        self.approximate_flops(capacity_gflops) / (capacity_gflops * 1_000_000_000.0) * 100.0
    }
}
impl LoadHold {
    pub fn validate(&self) -> crate::Result<()> {
        self.target.validate()?;
        if self.threads == Some(0) {
            return Err(crate::Error::Probe("CPU worker count must be greater than zero".into()));
        }
        Ok(())
    }
}
impl LoadControl {
    pub(crate) fn measured(target: LoadTarget, measured: f64, duty: f64, generator_flops: f64) -> Self {
        let (mode, target_percent, measured_percent, target_flops, measured_flops) = match target {
            LoadTarget::Percent(value) => ("percent", Some(value), Some(measured), None, None),
            LoadTarget::Flops(value) => ("flops", None, None, Some(value), Some(measured)),
        };
        Self {
            mode: mode.into(),
            target_percent,
            measured_percent,
            target_flops,
            measured_flops,
            duty_percent: duty * 100.0,
            generator_flops,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{
        Message,
        operations::{OperationDeclaration, Visibility},
    };
    use dokime::telemetry::TelemetryRow;
    #[test]
    fn validates_percent_flops_and_workers() {
        for target in [LoadTarget::Percent(0.01), LoadTarget::Percent(100.0), LoadTarget::Flops(1.0)] {
            assert!(LoadHold { target, threads: None }.validate().is_ok());
            let request = LoadHold { target, threads: Some(1) };
            assert_eq!(request, LoadHold::decode(&request.encode()).unwrap());
            assert!(LoadHold { target, threads: Some(0) }.validate().is_err());
        }
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(LoadTarget::Percent(value).validate().is_err());
            assert!(LoadTarget::Flops(value).validate().is_err());
        }
        assert!(LoadTarget::Percent(100.01).validate().is_err());
        assert_eq!(LoadHold::ID, "host.cpu.load.hold");
        assert_eq!(LoadHold::VISIBILITY, Visibility::Public);
    }
    #[test]
    fn evidence_preserves_units_and_target_mode() {
        for target in [LoadTarget::Percent(30.0), LoadTarget::Flops(123.0)] {
            let row = LoadControl::measured(target, 20.0, 0.25, 123.0);
            assert_eq!(row.duty_percent, 25.0);
            assert_eq!(row.target_percent.is_some(), matches!(target, LoadTarget::Percent(_)));
            assert_eq!(row, LoadControl::decode(&row.encode()).unwrap());
        }
        for column in LoadControl::columns().into_iter().filter(|column| column.unit.is_some()) {
            let unit = column.unit.unwrap();
            assert!(column.metadata.label.ends_with(&format!("[{}]", unit.symbol)));
        }
        assert_eq!(LoadTarget::Percent(25.0).approximate_flops(4.0), 1_000_000_000.0);
        assert_eq!(LoadTarget::Flops(1_000_000_000.0).approximate_percent(4.0), 25.0);
    }
}

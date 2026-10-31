/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod controller;
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "runtime")]
mod workers;

/* === Re-Exports === */

pub use crate::settings::CpuLoadSettings;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

// Local Imports

/* === Constants === */

pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: crate::settings::CPU_LOAD_HOLD_BUDGET,
    attempt_timeout: crate::settings::CPU_LOAD_HOLD_BUDGET,
    cancellation_timeout: crate::settings::CPU_LOAD_CANCELLATION_GRACE,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "host.cpu.load.hold", response = LoadSummary, policy = POLICY, visibility = Public,
    resources = [tokio::sync::Mutex<super::super::system::State>, CpuLoadSettings],
    operations = [dokime::harmos::builtins::streams::Publish])]
#[derive(Clone, Debug, PartialEq)]
pub struct LoadHold {
    pub target: LoadTarget,
    pub threads: Option<u32>,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct LoadSummary {
    pub target: LoadTarget,
    pub mean: f64,
    pub error_rms: f64,
    pub duration_s: f64,
    pub samples: u64,
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
    pub fn worker_count(&self, logical_cores: usize) -> crate::Result<usize> {
        self.validate()?;
        let count = self.threads.map(|count| count as usize).unwrap_or(logical_cores);
        if count == 0 || count > logical_cores {
            return Err(crate::Error::Probe("CPU worker count must fit the available logical cores".into()));
        }
        Ok(count)
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.target.validate()?;
        if self.threads == Some(0) {
            return Err(crate::Error::Probe("CPU worker count must be greater than zero".into()));
        }
        Ok(())
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
        let request = LoadHold {
            target: LoadTarget::Percent(50.0),
            threads: None,
        };
        assert_eq!(request.worker_count(4).unwrap(), 4);
        assert_eq!(LoadHold { threads: Some(2), ..request.clone() }.worker_count(4).unwrap(), 2);
        assert!(LoadHold { threads: Some(5), ..request.clone() }.worker_count(4).is_err());
        assert!(LoadHold { threads: Some(u32::MAX), ..request }.worker_count(4).is_err());
        assert!(LoadTarget::Percent(100.01).validate().is_err());
        assert_eq!(LoadHold::ID, "host.cpu.load.hold");
        assert_eq!(LoadHold::VISIBILITY, Visibility::Public);
    }
}

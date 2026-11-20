/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CPU Load Hold: Drive Workers to a Utilization or Throughput Target Until Cancelled

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;
use std::time::Instant;

// Third Party Imports

// Local Imports
use super::controller::{Controller, Summary};
use super::sampling::Source;
use super::workers::Workers;
use crate::domain::telemetry::{LoadFlopsControl, LoadPercentControl};
use crate::settings::CpuLoad;
use dokime::{Error, Result};

/* === Constants === */

const FLOPS_PER_GIGAFLOP: f64 = 1_000_000_000.0;
const PERCENT_SCALE: f64 = 100.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Hold a Percent or FLOP/s Target, Exactly One of Them, on Some or All Logical Cores.
#[derive(dokime::Message)]
#[dokime(package = "host")]
pub struct LoadHold {
    pub target_percent: Option<f64>,
    pub target_flops: Option<f64>,
    pub threads: Option<u32>,
}

/// The Time-Weighted Outcome of a Hold.
#[derive(dokime::Message)]
#[dokime(package = "host")]
pub struct LoadSummary {
    pub target_percent: Option<f64>,
    pub target_flops: Option<f64>,
    pub mean: f64,
    pub error_rms: f64,
    pub duration_s: f64,
    pub samples: u64,
}

/* === Enums === */

/// One Controller Period's Row, in the Stream of the Hold's Target Mode.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadControl {
    Percent(LoadPercentControl),
    Flops(LoadFlopsControl),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadTarget {
    Percent(f64),
    Flops(f64),
}

/* === Traits === */

/* === Implementations === */

impl LoadHold {
    pub fn target(&self) -> Result<LoadTarget> {
        let target = match (self.target_percent, self.target_flops) {
            (Some(value), None) => LoadTarget::Percent(value),
            (None, Some(value)) => LoadTarget::Flops(value),
            _ => return Err(Error::Invalid("Hold CPU Load: Request Exactly One of Target Percent and Target FLOP/s".into())),
        };
        target.validate()?;
        Ok(target)
    }

    pub fn worker_count(&self, logical_cores: usize) -> Result<usize> {
        self.target()?;
        let count = self.threads.map(|count| count as usize).unwrap_or(logical_cores);
        if count == 0 || count > logical_cores {
            return Err(Error::Invalid(format!("Hold CPU Load: Request From 1 to {logical_cores} Threads")));
        }
        Ok(count)
    }
}

impl LoadSummary {
    pub fn new(target: LoadTarget, mean: f64, error_rms: f64, duration_s: f64, samples: u64) -> Self {
        let (target_percent, target_flops) = target.split();
        Self {
            target_percent,
            target_flops,
            mean,
            error_rms,
            duration_s,
            samples,
        }
    }
}

impl LoadControl {
    pub fn measured(target: LoadTarget, measured: f64, duty: f64, generator_flops: f64) -> Self {
        let duty_percent = duty * PERCENT_SCALE;
        match target {
            LoadTarget::Percent(target_percent) => Self::Percent(LoadPercentControl {
                target_percent,
                measured_percent: measured,
                duty_percent,
                generator_flops,
            }),
            LoadTarget::Flops(target_flops) => Self::Flops(LoadFlopsControl {
                target_flops,
                measured_flops: measured,
                duty_percent,
                generator_flops,
            }),
        }
    }
}

impl LoadTarget {
    pub fn validate(self) -> Result<()> {
        let valid = match self {
            Self::Percent(value) => value.is_finite() && value > 0.0 && value <= PERCENT_SCALE,
            Self::Flops(value) => value.is_finite() && value > 0.0,
        };
        if !valid {
            return Err(Error::Invalid("Hold CPU Load: Request a Percent in (0, 100] or a Positive Finite FLOP/s Target".into()));
        }
        Ok(())
    }

    pub fn value(self) -> f64 {
        match self {
            Self::Percent(value) | Self::Flops(value) => value,
        }
    }

    pub fn approximate_flops(self, capacity_gflops: f64) -> f64 {
        match self {
            Self::Percent(value) => capacity_gflops * FLOPS_PER_GIGAFLOP * value / PERCENT_SCALE,
            Self::Flops(value) => value,
        }
    }

    pub fn approximate_percent(self, capacity_gflops: f64) -> f64 {
        self.approximate_flops(capacity_gflops) / (capacity_gflops * FLOPS_PER_GIGAFLOP) * PERCENT_SCALE
    }

    fn split(self) -> (Option<f64>, Option<f64>) {
        match self {
            Self::Percent(value) => (Some(value), None),
            Self::Flops(value) => (None, Some(value)),
        }
    }
}

/* === Public Functions === */

/// Hold the Target Until `cancel` Completes, Handing Each Controller Period's Row to `publish`.
pub async fn hold<C, F, P>(request: LoadHold, settings: CpuLoad, source: Source, cancel: C, mut publish: F) -> Result<LoadSummary>
where
    C: Future<Output = ()>,
    F: FnMut(LoadControl) -> P,
    P: Future<Output = Result<()>>,
{
    let target = request.target()?;
    settings.validate()?;
    let logical =
        std::thread::available_parallelism().map_err(|error| Error::Invalid(format!("Hold CPU Load: {error}; Restart the Plugin")))?.get();
    let count = request.worker_count(logical)?;
    let capacity = source.read().capacity_gflops * FLOPS_PER_GIGAFLOP;
    let scale = match target {
        LoadTarget::Percent(_) => PERCENT_SCALE * count as f64 / logical as f64,
        LoadTarget::Flops(_) => capacity * count as f64 / logical as f64,
    };
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::Invalid("Hold CPU Load: the Calibrated Capacity Is Not Positive; Restart the Plugin to Recalibrate".into()));
    }
    let mut summary = Summary::new(target);
    tokio::pin!(cancel);
    tokio::select! {
        biased;
        () = &mut cancel => return Ok(summary.finish()),
        () = std::future::ready(()) => {},
    }

    // Start Workers
    let mut controller = Controller::new(&settings);
    let mut workers = Workers::start(count, settings.slice, settings.batch_iterations)?;
    let initial = measured(target, source.read().utilization_percent, 0.0);
    let mut duty = controller.update(normalized_error(target, initial, scale), settings.period.as_secs_f64());
    workers.duty(duty);
    let mut previous = Instant::now();
    let mut operations = workers.operations();

    // Control Each Period Until Cancelled
    let outcome = loop {
        tokio::select! {
            biased;
            () = &mut cancel => break Ok(()),
            () = tokio::time::sleep_until((previous + settings.period).into()) => {},
        }
        let now = Instant::now();
        let elapsed = now.duration_since(previous).as_secs_f64();
        let completed = workers.operations();
        let flops = completed.wrapping_sub(operations) as f64 / elapsed;
        let value = measured(target, source.read().utilization_percent, flops);
        summary.record(value, elapsed);
        let row = LoadControl::measured(target, value, duty, flops);
        duty = controller.update(normalized_error(target, value, scale), elapsed);
        workers.duty(duty);
        previous = now;
        operations = completed;
        let published = tokio::select! {
            biased;
            () = &mut cancel => break Ok(()),
            result = publish(row) => result,
        };
        if let Err(error) = published {
            break Err(error);
        }
    };

    // Settle Workers
    let stopping = Instant::now();
    workers.stop()?;
    if stopping.elapsed() > settings.settlement {
        return Err(Error::Invalid("Hold CPU Load: Workers Exceeded the Settlement Budget; Lengthen the Worker Settlement Budget".into()));
    }
    outcome?;
    let elapsed = previous.elapsed().as_secs_f64();
    let flops = workers.operations().wrapping_sub(operations) as f64 / elapsed;
    summary.record(measured(target, source.read().utilization_percent, flops), elapsed);
    Ok(summary.finish())
}

/* === Private Functions === */

fn normalized_error(target: LoadTarget, measured: f64, scale: f64) -> f64 {
    (target.value() / scale - measured / scale).clamp(-1.0, 1.0)
}

fn measured(target: LoadTarget, utilization: f64, flops: f64) -> f64 {
    match target {
        LoadTarget::Percent(_) => utilization,
        LoadTarget::Flops(_) => flops,
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{LoadControl, LoadHold, LoadTarget, hold};
    use crate::collectors::cpu::Load;
    use crate::collectors::cpu::controller::tests::load_settings;
    use crate::collectors::cpu::sampling::Source;

    fn request(target_percent: Option<f64>, target_flops: Option<f64>, threads: Option<u32>) -> LoadHold {
        LoadHold { target_percent, target_flops, threads }
    }

    #[test]
    fn validates_targets_and_workers() {
        for (percent, flops) in [(Some(0.01), None), (Some(100.0), None), (None, Some(1.0))] {
            assert!(request(percent, flops, None).target().is_ok());
            assert!(request(percent, flops, Some(0)).worker_count(4).is_err());
        }
        for value in [0.0, -1.0, 100.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(request(Some(value), None, None).target().is_err());
        }
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(request(None, Some(value), None).target().is_err());
        }
        assert!(request(None, None, None).target().is_err());
        assert!(request(Some(50.0), Some(1.0), None).target().is_err());
        assert_eq!(request(Some(50.0), None, None).worker_count(4).unwrap(), 4);
        assert_eq!(request(Some(50.0), None, Some(2)).worker_count(4).unwrap(), 2);
        assert!(request(Some(50.0), None, Some(5)).worker_count(4).is_err());
        assert!(request(Some(50.0), None, Some(u32::MAX)).worker_count(4).is_err());
    }

    #[test]
    fn control_rows_carry_only_the_target_mode() {
        for target in [LoadTarget::Percent(30.0), LoadTarget::Flops(123.0)] {
            let duty_percent = match (LoadControl::measured(target, 20.0, 0.25, 123.0), target) {
                (LoadControl::Percent(row), LoadTarget::Percent(30.0)) => row.duty_percent,
                (LoadControl::Flops(row), LoadTarget::Flops(123.0)) => row.duty_percent,
                (row, target) => panic!("{target:?} Published {row:?}"),
            };
            assert_eq!(duty_percent, 25.0);
        }
        assert_eq!(LoadTarget::Percent(25.0).approximate_flops(4.0), 1_000_000_000.0);
        assert_eq!(LoadTarget::Flops(1_000_000_000.0).approximate_percent(4.0), 25.0);
    }

    #[tokio::test]
    async fn an_already_cancelled_hold_starts_no_workers() {
        let source = Source::fixed(Load::measured(1.0, 25.0));
        let summary = hold(request(Some(40.0), None, Some(1)), load_settings(), source, std::future::ready(()), |_| async {
            panic!("A Cancelled Hold Published a Period")
        })
        .await
        .unwrap();
        assert_eq!(summary.duration_s, 0.0);
        assert_eq!(summary.samples, 0);
        assert_eq!(summary.target_percent, Some(40.0));
    }
}

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Closed-Loop CPU Load Controller and the Time-Weighted Summary of a Hold

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::hold::{LoadSummary, LoadTarget};
use crate::settings::CpuLoad;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Proportional-Integral Controller With Anti-Windup, Answering a Duty in [0, 1].
pub struct Controller {
    proportional_gain: f64,
    integral_gain: f64,
    integral_min: f64,
    integral_max: f64,
    integral: f64,
}

pub struct Summary {
    target: LoadTarget,
    mean: f64,
    error_rms: f64,
    elapsed: f64,
    samples: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Controller {
    pub fn new(settings: &CpuLoad) -> Self {
        Self {
            proportional_gain: settings.proportional_gain,
            integral_gain: settings.integral_gain,
            integral_min: settings.integral_min,
            integral_max: settings.integral_max,
            integral: 0.0,
        }
    }

    pub fn update(&mut self, error: f64, elapsed_s: f64) -> f64 {
        let proportional = self.proportional_gain * error;
        self.integral = (self.integral + self.integral_gain * error * elapsed_s).clamp(self.integral_min, self.integral_max);
        // Back-Calculate at Saturation as Well as Clamping the Integrator.
        let output = proportional + self.integral;
        let duty = output.clamp(0.0, 1.0);
        if output != duty {
            self.integral = (duty - proportional).clamp(self.integral_min, self.integral_max);
        }
        duty
    }
}

impl Summary {
    pub fn new(target: LoadTarget) -> Self {
        Self {
            target,
            mean: 0.0,
            error_rms: 0.0,
            elapsed: 0.0,
            samples: 0,
        }
    }

    pub fn record(&mut self, measured: f64, elapsed_s: f64) {
        if elapsed_s <= 0.0 {
            return;
        }
        let elapsed = self.elapsed + elapsed_s;
        let weight = elapsed_s / elapsed;
        self.mean += (measured - self.mean) * weight;
        self.error_rms = (self.error_rms * (1.0 - weight).sqrt()).hypot((measured - self.target.value()).abs() * weight.sqrt());
        self.elapsed = elapsed;
        self.samples += 1;
    }

    pub fn finish(self) -> LoadSummary {
        LoadSummary::new(self.target, self.mean, if self.elapsed > 0.0 { self.error_rms } else { self.target.value() }, self.elapsed, self.samples)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;

    use super::{Controller, LoadTarget, Summary};
    use crate::collectors::cpu::Load;
    use crate::collectors::cpu::sampling::Source;
    use crate::settings::CpuLoad;

    /// Controller and Worker Parameters for Deterministic Tests.
    pub(crate) fn load_settings() -> CpuLoad {
        CpuLoad {
            period: Duration::from_millis(250),
            sample_period: Duration::from_millis(250),
            slice: Duration::from_millis(10),
            proportional_gain: 0.5,
            integral_gain: 0.5,
            integral_min: -1.0,
            integral_max: 1.0,
            settlement: Duration::from_secs(2),
            batch_iterations: 4_096,
        }
    }

    #[test]
    fn simulated_plant_converges_in_both_modes() {
        let settings = load_settings();
        for (target, capacity, background) in [(LoadTarget::Percent(60.0), 100.0, 20.0), (LoadTarget::Flops(400.0), 1000.0, 0.0)] {
            let source = Source::fixed(Load::measured(capacity / 1_000_000_000.0, background));
            let mut controller = Controller::new(&settings);
            let mut plant = background;
            let mut flops = 0.0;
            for _ in 0..400 {
                let measured = match target {
                    LoadTarget::Percent(_) => source.read().utilization_percent,
                    LoadTarget::Flops(_) => flops,
                };
                let duty = controller.update((target.value() - measured) / capacity, settings.period.as_secs_f64());
                plant += (background + capacity * duty - plant) * 0.4;
                flops = plant;
                source.update(Load::measured(capacity / 1_000_000_000.0, plant));
            }
            assert!((plant - target.value()).abs() < capacity * 0.01, "{target:?}: {plant}");
        }
    }

    #[test]
    fn saturation_is_bounded_and_recovers() {
        let settings = load_settings();
        let mut controller = Controller::new(&settings);
        for _ in 0..1000 {
            assert_eq!(controller.update(10.0, 1.0), 1.0);
            assert!((settings.integral_min..=settings.integral_max).contains(&controller.integral));
        }
        assert_eq!(controller.update(-10.0, 1.0), 0.0);
        let mut measured = 0.0;
        for _ in 0..400 {
            let duty = controller.update(0.4 - measured, settings.period.as_secs_f64());
            measured += (duty - measured) * 0.4;
        }
        assert!((measured - 0.4).abs() < 0.01);
    }

    #[test]
    fn summary_weights_actual_elapsed_time() {
        let mut summary = Summary::new(LoadTarget::Percent(50.0));
        summary.record(40.0, 1.0);
        summary.record(60.0, 3.0);
        let result = summary.finish();
        assert_eq!(result.mean, 55.0);
        assert!((result.error_rms - 10.0).abs() < 0.000_001);
        assert_eq!(result.duration_s, 4.0);
        assert_eq!(result.samples, 2);
        assert!(Summary::new(LoadTarget::Flops(1.0)).finish().error_rms.is_finite());
        let mut extreme = Summary::new(LoadTarget::Flops(f64::MAX));
        extreme.record(0.0, 1.0);
        assert_eq!(extreme.finish().error_rms, f64::MAX);
    }
}

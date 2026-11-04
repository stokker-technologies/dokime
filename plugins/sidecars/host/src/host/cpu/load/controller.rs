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

// Third Party Imports

// Local Imports
use super::{CpuLoadSettings, LoadSummary, LoadTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Controller {
    settings: CpuLoadSettings,
    integral: f64,
}
pub(super) struct Summary {
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
    pub(super) fn new(settings: CpuLoadSettings) -> Self {
        Self { settings, integral: 0.0 }
    }
    pub(super) fn update(&mut self, error: f64, elapsed_s: f64) -> f64 {
        let proportional = self.settings.proportional_gain * error;
        self.integral =
            (self.integral + self.settings.integral_gain * error * elapsed_s).clamp(self.settings.integral_min, self.settings.integral_max);
        // Back-Calculate At Saturation As Well As Clamping The Integrator.
        let output = proportional + self.integral;
        let duty = output.clamp(0.0, 1.0);
        if output != duty {
            self.integral = (duty - proportional).clamp(self.settings.integral_min, self.settings.integral_max);
        }
        duty
    }
}
impl Summary {
    pub(super) fn new(target: LoadTarget) -> Self {
        Self {
            target,
            mean: 0.0,
            error_rms: 0.0,
            elapsed: 0.0,
            samples: 0,
        }
    }
    pub(super) fn record(&mut self, measured: f64, elapsed_s: f64) {
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
    pub(super) fn finish(self) -> LoadSummary {
        LoadSummary {
            target: self.target,
            mean: self.mean,
            error_rms: if self.elapsed > 0.0 { self.error_rms } else { self.target.value() },
            duration_s: self.elapsed,
            samples: self.samples,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::CPU_LOAD_DEFAULTS as DEFAULT;
    #[test]
    fn simulated_plant_converges_in_both_modes() {
        use crate::host::cpu::{
            Load,
            sampling::tests::{fixed, update},
        };
        for (target, capacity, background) in [(LoadTarget::Percent(60.0), 100.0, 20.0), (LoadTarget::Flops(400.0), 1000.0, 0.0)] {
            let sampler = fixed(Load::measured(capacity / 1_000_000_000.0, background));
            let source = sampler.source();
            let mut controller = Controller::new(DEFAULT);
            let mut plant = background;
            let mut flops = 0.0;
            for _ in 0..400 {
                let measured = match target {
                    LoadTarget::Percent(_) => source.read().unwrap().utilization_percent,
                    LoadTarget::Flops(_) => flops,
                };
                let duty = controller.update((target.value() - measured) / capacity, DEFAULT.period().as_secs_f64());
                plant += (background + capacity * duty - plant) * 0.4;
                flops = plant;
                update(&sampler, Load::measured(capacity / 1_000_000_000.0, plant));
            }
            assert!((plant - target.value()).abs() < capacity * 0.01, "{target:?}: {plant}");
        }
    }
    #[test]
    fn saturation_is_bounded_and_recovers() {
        let mut controller = Controller::new(DEFAULT);
        for _ in 0..1000 {
            assert_eq!(controller.update(10.0, 1.0), 1.0);
            assert!((DEFAULT.integral_min..=DEFAULT.integral_max).contains(&controller.integral));
        }
        assert_eq!(controller.update(-10.0, 1.0), 0.0);
        let mut measured = 0.0;
        for _ in 0..400 {
            let duty = controller.update(0.4 - measured, DEFAULT.period().as_secs_f64());
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

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
use dokime::analysis::{
    Analysis, Limits, Operation, Output, Response, ResultData, Samples, Values,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct PeakToPeak;
pub(crate) struct Amplitude {
    value: f64,
    unit: String,
}

/* === Enums === */
#[dokime::catalog]
pub(crate) enum Analyses {
    PeakToPeak(Operation<PeakToPeak>),
}

/* === Traits === */

/* === Implementations === */
impl Analysis for PeakToPeak {
    type Parameters = ();
    type Output = Amplitude;
    type Error = std::convert::Infallible;
    const ID: &'static str = "route-fixture.peak-to-peak";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Peak To Peak";
    const DESCRIPTION: &'static str = "Maximum minus minimum over original samples.";
    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
    const LIMITS: Limits = Limits {
        max_samples: 1024,
        max_output_values: 1,
    };
    fn evaluate(samples: &Samples, _: &()) -> Result<Amplitude, Self::Error> {
        let minimum = samples.values.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum = samples
            .values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        Ok(Amplitude {
            value: maximum - minimum,
            unit: samples.unit.clone(),
        })
    }
}
impl ResultData for Amplitude {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "amplitude".into(),
            label: "Amplitude".into(),
            unit: None,
            axis: None,
        }]
    }
    fn into_response(self) -> Response {
        Response {
            outputs: vec![Values {
                key: "amplitude".into(),
                unit: self.unit,
                axis_unit: String::new(),
                axis: vec![],
                values: vec![self.value],
            }],
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

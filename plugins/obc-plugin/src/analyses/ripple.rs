/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::convert::Infallible;
use std::time::Duration;

use dokime::analysis::{Analysis, Limits, Output, Response, ResultData, Samples, Values};

pub struct Ripple;

pub struct Amplitude {
    value: f64,
    unit: String,
}

impl Analysis for Ripple {
    type Parameters = ();
    type Output = Amplitude;
    type Error = Infallible;
    const ID: &'static str = "obc.analysis.ripple";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Ripple";
    const DESCRIPTION: &'static str = "Peak-to-peak ripple: the maximum minus the minimum of the trace's original samples.";
    const TIMEOUT: Duration = Duration::from_secs(5);
    const LIMITS: Limits = Limits { max_samples: 1024, max_output_values: 1 };

    fn evaluate(samples: &Samples, _: &()) -> Result<Amplitude, Infallible> {
        let values = samples.values.iter().copied();
        let minimum = values.clone().fold(f64::INFINITY, f64::min);
        let maximum = values.fold(f64::NEG_INFINITY, f64::max);

        Ok(Amplitude {
            value: maximum - minimum,
            unit: samples.unit.clone(),
        })
    }
}

impl ResultData for Amplitude {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "ripple".into(),
            label: "Ripple".into(),
            unit: None,
            axis: None,
        }]
    }

    fn into_response(self) -> Response {
        Response {
            outputs: vec![Values {
                key: "ripple".into(),
                unit: self.unit,
                axis_unit: String::new(),
                axis: vec![],
                values: vec![self.value],
            }],
        }
    }
}

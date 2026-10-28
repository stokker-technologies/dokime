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
use std::{sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use crate::analysis::{
    Analysis, Axis, Definition, Definitions, Error, Handler, Limits, MAX_SAMPLES, Output, Response, ResultData, Samples, Values,
};
use crate::catalog::Catalog;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Mean;

struct Lttb;

struct Extrema;

#[cfg(feature = "fft")]
struct Fft;

struct Scalar {
    value: f64,
    unit: String,
}

struct Curve(Values);

#[cfg(feature = "fft")]
struct Spectrum(Values);

#[crate::parameters]
struct Reduction {
    #[parameter(label = "Points", default = 2500, min = 3, max = 262144)]
    points: u64,
}

#[cfg(feature = "fft")]
#[crate::parameters]
struct Fourier {
    #[parameter(label = "Hann Window", default = true)]
    hann: bool,

    #[parameter(label = "Remove Mean", default = true)]
    detrend: bool,

    #[parameter(label = "Include DC", default = false)]
    include_dc: bool,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Mean(Handler<Mean>),
    Lttb(Handler<Lttb>),
    Extrema(Handler<Extrema>),
}

#[cfg(feature = "fft")]
#[harmos::catalog]
enum FourierOperations {
    Fft(Handler<Fft>),
}

/* === Traits === */

/* === Implementations === */

impl ResultData for Scalar {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "value".into(),
            label: "Value".into(),
            unit: None,
            axis: None,
        }]
    }
    fn into_response(self) -> Response {
        Response {
            outputs: vec![Values {
                key: "value".into(),
                unit: self.unit,
                values: vec![self.value],
                ..Default::default()
            }],
        }
    }
}

impl ResultData for Curve {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "signal".into(),
            label: "Signal".into(),
            unit: None,
            axis: Some(Axis { key: "source".into(), unit: None }),
        }]
    }
    fn into_response(self) -> Response {
        Response { outputs: vec![self.0] }
    }
}

#[cfg(feature = "fft")]
impl ResultData for Spectrum {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "spectrum".into(),
            label: "Spectrum".into(),
            unit: None,
            axis: Some(Axis {
                key: "frequency".into(),
                unit: Some("Hz".into()),
            }),
        }]
    }
    fn into_response(self) -> Response {
        Response { outputs: vec![self.0] }
    }
}

impl Analysis for Mean {
    type Parameters = ();
    type Output = Scalar;
    type Error = Error;
    const ID: &'static str = "dokime.analysis.mean";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Mean";
    const DESCRIPTION: &'static str = "Arithmetic Mean Of Original Samples";
    const TIMEOUT: Duration = Duration::from_secs(10);
    const LIMITS: Limits = Limits {
        max_samples: MAX_SAMPLES,
        max_output_values: 1,
    };
    fn evaluate(samples: &Samples, _: &()) -> Result<Scalar, Error> {
        // Divide Before Summation To Avoid Overflow Of Finite Same-Sign Inputs.
        let count = samples.values.len() as f64;
        let value = samples.values.iter().map(|value| value / count).sum();
        Ok(Scalar { value, unit: samples.unit.clone() })
    }
}

impl Analysis for Lttb {
    type Parameters = Reduction;
    type Output = Curve;
    type Error = Error;
    const ID: &'static str = "dokime.analysis.lttb";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "LTTB Downsampling";
    const DESCRIPTION: &'static str = "Select Original Samples Preserving Line Shape";
    const TIMEOUT: Duration = Duration::from_secs(10);
    const LIMITS: Limits = Limits {
        max_samples: MAX_SAMPLES,
        max_output_values: MAX_SAMPLES,
    };
    fn evaluate(samples: &Samples, parameters: &Reduction) -> Result<Curve, Error> {
        let indexes = harmos::signal::lttb(&samples.axis, &samples.values, parameters.points as usize);
        Ok(selected(samples, indexes))
    }
}

impl Analysis for Extrema {
    type Parameters = Reduction;
    type Output = Curve;
    type Error = Error;
    const ID: &'static str = "dokime.analysis.extrema";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Min/Max Downsampling";
    const DESCRIPTION: &'static str = "Select Original Bucket Extrema In Source Order";
    const TIMEOUT: Duration = Lttb::TIMEOUT;
    const LIMITS: Limits = Lttb::LIMITS;
    fn evaluate(samples: &Samples, parameters: &Reduction) -> Result<Curve, Error> {
        let indexes = if samples.values.len() <= parameters.points as usize {
            (0..samples.values.len()).collect()
        } else {
            harmos::signal::extrema(&samples.values, parameters.points as usize / 2)
        };
        Ok(selected(samples, indexes))
    }
}

#[cfg(feature = "fft")]
impl Analysis for Fft {
    type Parameters = Fourier;
    type Output = Spectrum;
    type Error = Error;
    const ID: &'static str = "dokime.analysis.fft";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "FFT Spectrum";
    const DESCRIPTION: &'static str = "One-Sided Amplitude Spectrum Of Uniform Time Samples";
    const TIMEOUT: Duration = Lttb::TIMEOUT;
    const LIMITS: Limits = Lttb::LIMITS;
    fn evaluate(samples: &Samples, parameters: &Fourier) -> Result<Spectrum, Error> {
        use super::{PlotPayloadRequest, PlotPoint, PlotSeries};
        use crate::document::{FftDetrend, FftOptions, FftWindow};
        if samples.axis_unit != "s" {
            return Err(Error::Input("FFT requires a time axis in seconds".into()));
        }
        let mut series = PlotSeries {
            key: "spectrum".into(),
            label: "Spectrum".into(),
            unit: Some(samples.unit.clone()),
            points: samples.axis.iter().zip(&samples.values).map(|(&x, &y)| PlotPoint { x: x.into(), y: Some(y), z: None }).collect(),
            raw_points: samples.values.len(),
            downsampled: false,
            format: None,
        };
        let request = PlotPayloadRequest {
            fft: FftOptions {
                window: if parameters.hann { FftWindow::Hann } else { FftWindow::Rectangular },
                detrend: if parameters.detrend { FftDetrend::Mean } else { FftDetrend::None },
                include_dc: parameters.include_dc,
                ..Default::default()
            },
            ..Default::default()
        };
        super::analyses::fft(&mut series, &request).map_err(Error::Input)?;
        Ok(Spectrum(Values {
            key: "spectrum".into(),
            unit: samples.unit.clone(),
            axis_unit: "Hz".into(),
            axis: series.points.iter().map(|point| point.x.as_f64().expect("numeric spectrum axis")).collect(),
            values: series.points.iter().map(|point| point.y.expect("finite spectrum amplitude")).collect(),
        }))
    }
}

/* === Public Functions === */

pub(crate) fn definitions() -> Result<Vec<Definition>, Error> {
    let entries = <Operations as Catalog<Definitions>>::ENTRIES;
    #[cfg(feature = "fft")]
    let entries = entries.into_iter().chain(<FourierOperations as Catalog<Definitions>>::ENTRIES);
    #[cfg(not(feature = "fft"))]
    let entries = entries.into_iter();
    entries.map(|entry| entry()).collect()
}

pub(crate) fn evaluate_report(operation: &str, request: crate::analysis::Request) -> Result<Response, Error> {
    match operation {
        Mean::ID => crate::analysis::Operation::<Mean>::evaluate(request),
        Lttb::ID => crate::analysis::Operation::<Lttb>::evaluate(request),
        Extrema::ID => crate::analysis::Operation::<Extrema>::evaluate(request),
        #[cfg(feature = "fft")]
        Fft::ID => crate::analysis::Operation::<Fft>::evaluate(request),
        _ => Err(Error::Input("report builtin analysis is unavailable".into())),
    }
}

pub(crate) fn register(operations: &harmos::operations::OperationManager) -> harmos::Result<()> {
    operations.register_wire_operations::<Operations, _>("dokime.analysis", crate::app::Lanes::Open, Arc::new(()))?;
    #[cfg(feature = "fft")]
    operations.register_wire_operations::<FourierOperations, _>("dokime.analysis", crate::app::Lanes::Open, Arc::new(()))?;
    Ok(())
}

/* === Private Functions === */

fn selected(samples: &Samples, indexes: Vec<usize>) -> Curve {
    Curve(Values {
        key: "signal".into(),
        unit: samples.unit.clone(),
        axis_unit: samples.axis_unit.clone(),
        axis: indexes.iter().map(|&index| samples.axis[index]).collect(),
        values: indexes.iter().map(|&index| samples.values[index]).collect(),
    })
}

/* === Tests === */

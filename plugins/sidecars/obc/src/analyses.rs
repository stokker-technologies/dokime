/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Ripple Analysis Any Plotted Trace Can Request

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::catalog::metadata;
use dokime::plugin::{AnalysesComputeRequest, AnalysesComputeResponse, Context};
use dokime::{Analysis, AnalysisOutput, AnalysisValues, Error, Result};

/* === Constants === */

pub const RIPPLE: &str = "obc.analysis.ripple";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn ripple() -> Analysis {
    Analysis {
        metadata: Some(dokime::Metadata {
            description: "Peak-to-peak ripple: the maximum minus the minimum of the trace's original samples.".into(),
            ..metadata(RIPPLE, "Ripple")
        }),
        version: 1,
        outputs: vec![AnalysisOutput {
            metadata: Some(metadata("ripple", "Ripple")),
            ..Default::default()
        }],
        max_samples: 1024,
        max_output_values: 1,
        ..Default::default()
    }
}

/// The Maximum Minus the Minimum of the Supplied Samples, in Their Own Unit.
pub async fn compute_ripple(_: &Context, request: AnalysesComputeRequest) -> Result<AnalysesComputeResponse> {
    let samples = request.samples.ok_or_else(|| Error::Invalid("Compute Ripple: Supply the Trace Samples".into()))?;
    if samples.values.is_empty() {
        return Err(Error::Invalid("Compute Ripple: Select a Window With Samples".into()));
    }
    let minimum = samples.values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = samples.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Ok(AnalysesComputeResponse {
        outputs: vec![AnalysisValues {
            key: "ripple".into(),
            unit: samples.unit,
            values: vec![maximum - minimum],
            ..Default::default()
        }],
    })
}

/* === Private Functions === */

/* === Tests === */
